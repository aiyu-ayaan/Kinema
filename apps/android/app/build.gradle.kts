import java.util.Properties

plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.kotlin.compose)
    alias(libs.plugins.kotlin.serialization) // navigation keys are saved across process death
    alias(libs.plugins.baselineprofile) // ahead-of-time compiles the startup and scroll paths
}

// Release signing from gitignored keystore.properties; without it the release build is unsigned.
val keystoreProps = Properties().apply {
    rootProject.file("keystore.properties").takeIf { it.exists() }?.inputStream()?.use(::load)
}

// The app's name (`version`) and number (`build`) follow the root package.json, bumped by the release PR.
val rootManifest = groovy.json.JsonSlurper().parse(rootProject.file("../../package.json")) as Map<*, *>

android {
    namespace = "com.aiyu.kinema"
    compileSdk = 37

    defaultConfig {
        applicationId = "com.aiyu.kinema"
        minSdk = 26
        targetSdk = 36
        versionCode = rootManifest["build"] as Int
        versionName = rootManifest["version"] as String
    }

    flavorDimensions += "environment"
    productFlavors {
        create("dev") {
            dimension = "environment"
            applicationIdSuffix = ".dev"
            versionNameSuffix = "-dev"
            manifestPlaceholders["appName"] = "Kinema Dev"
        }
        create("prod") {
            dimension = "environment"
            manifestPlaceholders["appName"] = "Kinema"
        }
    }

    signingConfigs {
        keystoreProps.getProperty("storeFile")?.let { path ->
            create("release") {
                storeFile = rootProject.file(path)
                storePassword = keystoreProps.getProperty("storePassword")
                keyAlias = keystoreProps.getProperty("keyAlias")
                keyPassword = keystoreProps.getProperty("keyPassword")
            }
        }
    }

    buildTypes {
        release {
            isMinifyEnabled = true
            isShrinkResources = true
            signingConfig = signingConfigs.findByName("release")
            proguardFiles(getDefaultProguardFile("proguard-android-optimize.txt"))
        }
    }

    buildFeatures {
        compose = true
        buildConfig = true
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
}

baselineProfile {
    // The profile lands in src/main so every flavor and the release build ship it.
    mergeIntoMain = true
    saveInSrc = true
}

kotlin {
    compilerOptions {
        optIn.addAll(
            "androidx.compose.material3.ExperimentalMaterial3Api",
            "androidx.compose.material3.ExperimentalMaterial3ExpressiveApi",
            "androidx.media3.common.util.UnstableApi",
        )
    }
}

dependencies {
    implementation(project(":core"))
    implementation(libs.androidx.core.ktx)
    implementation(libs.androidx.activity.compose)
    implementation(libs.androidx.lifecycle.runtime.compose)
    implementation(libs.androidx.graphics.shapes)
    implementation(libs.navigation3.runtime)
    implementation(libs.navigation3.ui)
    implementation(libs.kotlinx.serialization.json)
    implementation(platform(libs.compose.bom))
    implementation(libs.compose.ui)
    implementation(libs.compose.ui.tooling.preview)
    implementation(libs.compose.material3)
    implementation(libs.compose.material.icons.extended)
    implementation(libs.media3.exoplayer)
    implementation(libs.media3.datasource.okhttp)
    implementation(libs.media3.ui.compose)
    implementation(libs.media3.session)
    implementation(libs.coil.compose)
    implementation(libs.coil.network.okhttp)
    implementation(libs.markdown.m3) // release notes on the Updates screen: headings, lists, tables
    implementation(libs.markdown.coil3) // images in release notes
    implementation(libs.profileinstaller) // installs the profile on first run, without waiting for the Play cloud profile
    "baselineProfile"(project(":baselineprofile"))
    debugImplementation(libs.compose.ui.tooling)
}
