package com.pocketclaw.app

import android.content.Intent
import android.graphics.Color
import android.graphics.Typeface
import android.os.Bundle
import android.view.Gravity
import android.widget.Button
import android.widget.LinearLayout
import android.widget.TextView
import android.widget.Toast
import androidx.appcompat.app.AppCompatActivity

class MainActivity : AppCompatActivity() {

    private lateinit var statusText: TextView

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        // Redirect to setup if no config exists
        if (!SetupActivity.hasConfig(this)) {
            startActivity(Intent(this, SetupActivity::class.java))
            finish()
            return
        }

        val layout = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            gravity = Gravity.CENTER
            setBackgroundColor(Color.parseColor("#1a1a2e"))
            setPadding(48, 48, 48, 48)
        }

        // Logo
        layout.addView(TextView(this).apply {
            text = "🦞"
            textSize = 64f
            gravity = Gravity.CENTER
            setPadding(0, 48, 0, 16)
        })

        // Title
        layout.addView(TextView(this).apply {
            text = "PocketClaw"
            textSize = 32f
            setTextColor(Color.WHITE)
            typeface = Typeface.DEFAULT_BOLD
            gravity = Gravity.CENTER
        })

        // Status
        statusText = TextView(this).apply {
            text = "Ready to start"
            textSize = 16f
            setTextColor(Color.parseColor("#aaaaaa"))
            gravity = Gravity.CENTER
            setPadding(0, 8, 0, 48)
        }
        layout.addView(statusText)

        // Start Button
        val startButton = Button(this).apply {
            text = "▶  Start Agent"
            textSize = 18f
            setTextColor(Color.WHITE)
            setBackgroundColor(Color.parseColor("#0f3460"))
            setPadding(48, 24, 48, 24)
            setOnClickListener {
                startService(Intent(this@MainActivity, PocketClawService::class.java))
                statusText.text = "Agent is running ✅"
                statusText.setTextColor(Color.parseColor("#00ff88"))
            }
        }
        layout.addView(startButton)

        // Spacer
        layout.addView(android.view.View(this).apply {
            layoutParams = LinearLayout.LayoutParams(
                LinearLayout.LayoutParams.MATCH_PARENT, 16
            )
        })

        // Stop Button
        val stopButton = Button(this).apply {
            text = "⏹  Stop Agent"
            textSize = 18f
            setTextColor(Color.WHITE)
            setBackgroundColor(Color.parseColor("#e94560"))
            setPadding(48, 24, 48, 24)
            setOnClickListener {
                val intent = Intent(this@MainActivity, PocketClawService::class.java)
                intent.action = "STOP"
                startService(intent)
                statusText.text = "Agent stopped"
                statusText.setTextColor(Color.parseColor("#aaaaaa"))
            }
        }
        layout.addView(stopButton)

        // Spacer
        layout.addView(android.view.View(this).apply {
            layoutParams = LinearLayout.LayoutParams(
                LinearLayout.LayoutParams.MATCH_PARENT, 32
            )
        })

        // Settings Button
        val settingsButton = Button(this).apply {
            text = "⚙  Settings"
            textSize = 14f
            setTextColor(Color.parseColor("#aaaaaa"))
            setBackgroundColor(Color.TRANSPARENT)
            setOnClickListener {
                startActivity(Intent(this@MainActivity, SetupActivity::class.java))
            }
        }
        layout.addView(settingsButton)

        setContentView(layout)
    }
}
