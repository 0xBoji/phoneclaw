package com.phoneclaw.app

object RustBridge {
    init {
        System.loadLibrary("mobile_jni")
    }

    external fun startServer(configPath: String): String
    external fun stopServer(): String

    @JvmStatic
    fun performClick(x: Float, y: Float): Boolean = false

    @JvmStatic
    fun performBack(): Boolean = false

    @JvmStatic
    fun performHome(): Boolean = false
    
    @JvmStatic
    fun performScroll(x1: Float, y1: Float, x2: Float, y2: Float): Boolean = false

    @JvmStatic
    fun performInputText(text: String): Boolean = false

    @JvmStatic
    fun performDumpHierarchy(): String = "<error>android control disabled</error>"

    @JvmStatic
    fun performTakeScreenshot(): ByteArray = ByteArray(0)
}
