plugins {
    kotlin("jvm") version "2.1.0"
}

// Le serveur gninou n'est pas une app Android classique : il est exécuté
// sur l'appareil via `app_process`. On compile donc un JAR Kotlin/JVM que
// l'on convertira ensuite en .dex (d8) pour le pousser via adb.

group = "io.karfi.gninou"
version = "0.1.0"

repositories {
    google()
    mavenCentral()
}

dependencies {
    // Les classes du framework Android (SurfaceControl, MediaCodec, ...) sont
    // fournies par le système au runtime : on compile contre un android.jar
    // (compileOnly). Renseigner le chemin via -PandroidJar ou local.properties.
    val androidJar = (findProperty("androidJar") as String?)
    if (androidJar != null) {
        compileOnly(files(androidJar))
    }

    testImplementation(kotlin("test"))
}

kotlin {
    jvmToolchain(17)
}

tasks.test {
    useJUnitPlatform()
}

tasks.jar {
    manifest {
        attributes("Main-Class" to "io.karfi.gninou.server.ServerKt")
    }
}
