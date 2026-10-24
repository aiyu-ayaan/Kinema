plugins {
    alias(libs.plugins.android.test)
    alias(libs.plugins.baselineprofile)
}

android {
    namespace = "com.aiyu.kinema.baselineprofile"
    compileSdk = 37

    defaultConfig {
        minSdk = 28 // Macrobenchmark's floor
        targetSdk = 36
        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    targetProjectPath = ":app"

    // Same flavors as the app. Generate with the prod one; class names match, and mergeIntoMain shares the profile.
    flavorDimensions += "environment"
    productFlavors {
        create("dev") {
            dimension = "environment"
            buildConfigField("String", "TARGET_APP_ID", "\"com.aiyu.kinema.dev\"")
        }
        create("prod") {
            dimension = "environment"
            buildConfigField("String", "TARGET_APP_ID", "\"com.aiyu.kinema\"")
        }
    }

    buildFeatures.buildConfig = true
}

baselineProfile {
    // Emulators run the generator too: nothing here needs a physical device.
    useConnectedDevices = true
}

dependencies {
    implementation(libs.test.ext.junit)
    implementation(libs.uiautomator)
    implementation(libs.benchmark.macro.junit4)
}
