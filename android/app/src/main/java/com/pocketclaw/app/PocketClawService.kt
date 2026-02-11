package com.pocketclaw.app

import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.app.Service
import android.content.Context
import android.content.Intent
import android.os.Build
import android.os.IBinder
import androidx.core.app.NotificationCompat
import java.io.File
import java.io.FileWriter

class PocketClawService : Service() {

    private val CHANNEL_ID = "PocketClawChannel"
    private var isRunning = false

    override fun onBind(intent: Intent): IBinder? {
        return null
    }

    override fun onCreate() {
        super.onCreate()
        createNotificationChannel()
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        if (intent?.action == "STOP") {
            stopSelf()
            return START_NOT_STICKY
        }

        if (!isRunning) {
            val notificationIntent = Intent(this, MainActivity::class.java)
            val pendingIntent = PendingIntent.getActivity(
                this, 0, notificationIntent, PendingIntent.FLAG_IMMUTABLE
            )

            val stopIntent = Intent(this, PocketClawService::class.java).apply {
                action = "STOP"
            }
            val stopPendingIntent = PendingIntent.getService(
                this, 0, stopIntent, PendingIntent.FLAG_IMMUTABLE
            )

            val notification = NotificationCompat.Builder(this, CHANNEL_ID)
                .setContentTitle("PocketClaw Agent")
                .setContentText("Running in background 🦞")
                .setSmallIcon(android.R.drawable.sym_def_app_icon)
                .setContentIntent(pendingIntent)
                .addAction(android.R.drawable.ic_menu_close_clear_cancel, "Stop", stopPendingIntent)
                .build()

            startForeground(1, notification)
            
            // Start Rust Server in a background thread
            Thread {
                isRunning = true
                val configPath = setupConfigFile()
                RustBridge.startServer(configPath)
                // When rust returns (if ever), stop service
                stopSelf()
            }.start()
        }

        return START_STICKY
    }

    private fun setupConfigFile(): String {
        // Create a basic config.json in app's internal storage if not exists
        val configDir = File(filesDir, ".pocketclaw")
        if (!configDir.exists()) {
            configDir.mkdirs()
        }
        val configFile = File(configDir, "config.json")
        /*
        if (!configFile.exists()) {
             // Create dummy config or copy from somewhere
             // For now, we assume user might edit it or we rely on onboarding logic inside Rust (which might fail without TTY)
             // Ideally we should copy a template from assets
        }
        */
        return configFile.absolutePath
    }

    private fun createNotificationChannel() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val serviceChannel = NotificationChannel(
                CHANNEL_ID,
                "PocketClaw Service Channel",
                NotificationManager.IMPORTANCE_DEFAULT
            )
            val manager = getSystemService(NotificationManager::class.java)
            manager.createNotificationChannel(serviceChannel)
        }
    }
}
