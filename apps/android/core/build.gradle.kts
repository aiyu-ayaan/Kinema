plugins {
    alias(libs.plugins.android.library)
    alias(libs.plugins.kotlin.serialization)
}

android {
    namespace = "com.aiyu.kinema.core"
    compileSdk = 37

    defaultConfig {
        minSdk = 26
        consumerProguardFiles("consumer-rules.pro")
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    testOptions {
        unitTests.isReturnDefaultValues = true
    }
}

dependencies {
    api(libs.kotlinx.coroutines.android)
    api(libs.okhttp) // the app hands this client to Coil and Media3, so images and video share its TLS pin and token
    implementation(libs.kotlinx.serialization.json)

    testImplementation(libs.junit)
}
