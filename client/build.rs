//! Script de build du client.
//!
//! Garantit la présence de `assets/gninou-server.jar`, que le client
//! embarque via `include_bytes!`. Deux cas :
//!   - `GNINOU_BUILD_SERVER=1` : compile le serveur Kotlin (Gradle) et copie
//!     le jar produit dans `assets/`.
//!   - sinon : si le jar est absent, crée un placeholder vide pour que la
//!     compilation aboutisse (l'app ne pourra pas pousser de serveur réel,
//!     c'est attendu pour un build de dev sans SDK Android).

use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let assets = Path::new("assets");
    let jar = assets.join("gninou-server.jar");
    std::fs::create_dir_all(assets).expect("création de assets/");

    if std::env::var("GNINOU_BUILD_SERVER").as_deref() == Ok("1") {
        if let Some(built) = build_server() {
            std::fs::copy(&built, &jar).expect("copie du jar serveur");
            println!("cargo:warning=serveur compilé et embarqué : {}", built.display());
        } else {
            println!("cargo:warning=échec du build serveur — placeholder conservé");
        }
    }

    if !jar.exists() {
        std::fs::write(&jar, b"").expect("écriture du placeholder");
        println!(
            "cargo:warning=assets/gninou-server.jar absent : placeholder vide utilisé. \
             Lancez un build avec GNINOU_BUILD_SERVER=1 (SDK Android requis) pour l'embarquer."
        );
    }

    println!("cargo:rerun-if-changed=assets/gninou-server.jar");
    println!("cargo:rerun-if-env-changed=GNINOU_BUILD_SERVER");
}

/// Compile le serveur via Gradle et renvoie le chemin du jar produit.
fn build_server() -> Option<PathBuf> {
    let server_dir = Path::new("..").join("server");
    if !server_dir.exists() {
        return None;
    }

    let gradlew = if cfg!(windows) { "gradlew.bat" } else { "./gradlew" };
    let program = if server_dir.join(gradlew).exists() {
        gradlew
    } else {
        "gradle" // repli sur un gradle système
    };

    let status = Command::new(program)
        .arg("jar")
        .current_dir(&server_dir)
        .status()
        .ok()?;

    if !status.success() {
        return None;
    }

    // build/libs/gninou-server-<version>.jar
    let libs = server_dir.join("build").join("libs");
    std::fs::read_dir(libs)
        .ok()?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .find(|p| p.extension().map(|x| x == "jar").unwrap_or(false))
}
