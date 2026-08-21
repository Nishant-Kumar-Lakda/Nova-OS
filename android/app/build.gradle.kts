plugins {
    id("com.android.application")
}

android {
    namespace = "org.nova.os"
    compileSdk = 35

    defaultConfig {
        applicationId = "org.nova.os"
        minSdk = 23
        targetSdk = 35
        versionCode = 1
        versionName = "0.1.0"
    }

    buildTypes {
        release {
            isMinifyEnabled = false
        }
        debug {
            isMinifyEnabled = false
        }
    }
}

dependencies {
    testImplementation("junit:junit:4.13.2")
}
