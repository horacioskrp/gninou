//! Reconnexion automatique avec backoff exponentiel (#31).
//!
//! Lorsqu'une connexion réseau (Wi-Fi/TCP-IP) échoue ou tombe, gninou peut
//! relancer l'opération (`adb connect`) automatiquement, en espaçant les essais
//! par un délai qui double à chaque fois jusqu'à un plafond.
//!
//! La **politique** ([`Backoff`] : calcul des délais, borne du nombre
//! d'essais) est pure et testée ; le **driver** [`retry`] qui patiente et
//! réexécute l'action est une fine couche d'I/O (son enchaînement est
//! néanmoins testé avec un délai nul).

use std::time::Duration;

use anyhow::{Context, Result};

/// Politique de reconnexion : délai de base, plafond, nombre total d'essais.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Backoff {
    /// Délai avant la **première** reprise, en millisecondes.
    pub base_ms: u64,
    /// Plafond du délai entre deux reprises, en millisecondes.
    pub max_ms: u64,
    /// Nombre total de tentatives, reprises comprises (≥ 1).
    pub max_attempts: u32,
}

impl Backoff {
    /// Politique explicite. `max_attempts` est ramené à 1 au minimum.
    pub const fn new(base_ms: u64, max_ms: u64, max_attempts: u32) -> Self {
        Backoff {
            base_ms,
            max_ms,
            max_attempts: if max_attempts == 0 { 1 } else { max_attempts },
        }
    }

    /// Délai (ms) avant la reprise n° `retry` (0 = première reprise).
    ///
    /// Croît en `base_ms * 2^retry`, plafonné à `max_ms`. Saturant : aucun
    /// débordement même pour un `retry` élevé. Logique **pure**.
    pub fn delay_ms(&self, retry: u32) -> u64 {
        let factor = 1u64.checked_shl(retry.min(63)).unwrap_or(u64::MAX);
        self.base_ms.saturating_mul(factor).min(self.max_ms)
    }

    /// Nombre de reprises autorisées (hors première tentative).
    pub fn retries(&self) -> u32 {
        self.max_attempts - 1
    }

    /// Délai avant la reprise n° `retry`, sous forme de [`Duration`].
    pub fn delay(&self, retry: u32) -> Duration {
        Duration::from_millis(self.delay_ms(retry))
    }
}

impl Default for Backoff {
    /// Valeurs par défaut raisonnables : 500 ms → 10 s, 10 essais.
    fn default() -> Self {
        Backoff::new(500, 10_000, 10)
    }
}

/// Exécute `action` jusqu'au succès ou épuisement des tentatives, en patientant
/// selon `policy` entre les essais. Renvoie la dernière erreur si tout échoue.
///
/// `sleep` reçoit chaque délai calculé : la production passe
/// `std::thread::sleep`, les tests un espion sans attente réelle.
pub fn retry_with<T, A, S>(policy: Backoff, mut action: A, mut sleep: S) -> Result<T>
where
    A: FnMut() -> Result<T>,
    S: FnMut(Duration),
{
    let mut last_err = None;
    for attempt in 1..=policy.max_attempts {
        if attempt > 1 {
            let delay = policy.delay(attempt - 2); // reprise 0 au 2ᵉ essai
            log::warn!(
                "reconnexion : tentative {}/{} dans {} ms",
                attempt,
                policy.max_attempts,
                delay.as_millis()
            );
            sleep(delay);
        }
        match action() {
            Ok(value) => {
                if attempt > 1 {
                    log::info!("reconnexion réussie (tentative {attempt})");
                }
                return Ok(value);
            }
            Err(e) => {
                log::debug!("tentative {attempt} échouée : {e}");
                last_err = Some(e);
            }
        }
    }
    Err(last_err.unwrap()).with_context(|| {
        format!(
            "reconnexion abandonnée après {} tentative(s)",
            policy.max_attempts
        )
    })
}

/// Comme [`retry_with`] mais en patientant réellement (`std::thread::sleep`).
pub fn retry<T, A>(policy: Backoff, action: A) -> Result<T>
where
    A: FnMut() -> Result<T>,
{
    retry_with(policy, action, std::thread::sleep)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    #[test]
    fn delai_exponentiel() {
        let b = Backoff::new(100, 100_000, 10);
        assert_eq!(b.delay_ms(0), 100);
        assert_eq!(b.delay_ms(1), 200);
        assert_eq!(b.delay_ms(2), 400);
        assert_eq!(b.delay_ms(3), 800);
    }

    #[test]
    fn delai_plafonne() {
        let b = Backoff::new(1000, 5000, 10);
        assert_eq!(b.delay_ms(0), 1000);
        assert_eq!(b.delay_ms(2), 4000);
        assert_eq!(b.delay_ms(3), 5000); // 8000 plafonné à 5000
        assert_eq!(b.delay_ms(50), 5000); // saturant, pas de débordement
    }

    #[test]
    fn au_moins_un_essai() {
        assert_eq!(Backoff::new(10, 20, 0).max_attempts, 1);
        assert_eq!(Backoff::new(10, 20, 1).retries(), 0);
    }

    #[test]
    fn reussit_du_premier_coup_sans_attente() {
        let slept = RefCell::new(Vec::new());
        let got = retry_with(
            Backoff::new(10, 100, 5),
            || Ok::<_, anyhow::Error>(42),
            |d| slept.borrow_mut().push(d),
        )
        .unwrap();
        assert_eq!(got, 42);
        assert!(slept.borrow().is_empty()); // aucun délai : succès immédiat
    }

    #[test]
    fn reussit_apres_deux_echecs() {
        let calls = RefCell::new(0);
        let slept = RefCell::new(Vec::new());
        let got = retry_with(
            Backoff::new(10, 1000, 5),
            || {
                let mut c = calls.borrow_mut();
                *c += 1;
                if *c < 3 {
                    anyhow::bail!("pas encore");
                }
                Ok(*c)
            },
            |d| slept.borrow_mut().push(d),
        )
        .unwrap();
        assert_eq!(got, 3); // réussit à la 3ᵉ tentative
                            // Deux attentes : avant la 2ᵉ (10 ms) et la 3ᵉ (20 ms).
        assert_eq!(
            *slept.borrow(),
            vec![Duration::from_millis(10), Duration::from_millis(20)]
        );
    }

    #[test]
    fn abandonne_apres_le_nombre_max() {
        let calls = RefCell::new(0);
        let res = retry_with(
            Backoff::new(0, 0, 3),
            || -> Result<()> {
                *calls.borrow_mut() += 1;
                anyhow::bail!("toujours ko")
            },
            |_d| {},
        );
        assert!(res.is_err());
        assert_eq!(*calls.borrow(), 3); // exactement max_attempts essais
    }
}
