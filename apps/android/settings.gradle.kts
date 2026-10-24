pluginManagement {
    repositories {
        google()
        mavenCentral()
        gradlePluginPortal()
    }
}
dependencyResolutionManagement {
    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)
    repositories {
        google()
        mavenCentral()
    }
}

rootProject.name = "Kinema"
include(":app")  // screens, navigation, player
include(":core") // server discovery, TLS trust, API client, session (no UI)
include(":baselineprofile") // generates the app's baseline profile and measures startup (not shipped)
