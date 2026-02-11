package com.pocketclaw.app

object RustBridge {
    init {
        System.loadLibrary("mobile_jni")
    }

    external fun startServer(configPath: String): String
    external fun stopServer(): String
}
