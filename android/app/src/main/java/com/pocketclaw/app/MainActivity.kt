package com.pocketclaw.app

import android.content.Intent
import android.graphics.Color
import android.graphics.Typeface
import android.os.Bundle
import android.os.Handler
import android.os.Looper
import android.view.Gravity
import android.widget.Button
import android.widget.LinearLayout
import android.widget.ScrollView
import android.widget.TextView
import androidx.appcompat.app.AppCompatActivity
import java.io.BufferedReader
import java.io.InputStreamReader

class MainActivity : AppCompatActivity() {

    private lateinit var statusText: TextView
    private lateinit var logView: TextView
    private lateinit var logScroll: ScrollView
    private var logThread: Thread? = null
    private var isLogging = false

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        // Redirect to setup if no config exists
        if (!SetupActivity.hasConfig(this)) {
            startActivity(Intent(this, SetupActivity::class.java))
            finish()
            return
        }

        val rootLayout = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            setBackgroundColor(Color.parseColor("#1a1a2e"))
        }

        // ─── Top Section ───
        val topLayout = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            gravity = Gravity.CENTER
            setPadding(48, 32, 48, 16)
        }

        topLayout.addView(TextView(this).apply {
            text = "🦞 PocketClaw"
            textSize = 28f
            setTextColor(Color.WHITE)
            typeface = Typeface.DEFAULT_BOLD
            gravity = Gravity.CENTER
        })

        statusText = TextView(this).apply {
            text = "Ready to start"
            textSize = 14f
            setTextColor(Color.parseColor("#aaaaaa"))
            gravity = Gravity.CENTER
            setPadding(0, 8, 0, 16)
        }
        topLayout.addView(statusText)

        // Buttons row
        val buttonRow = LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
            gravity = Gravity.CENTER
        }

        val startButton = Button(this).apply {
            text = "▶ Start"
            textSize = 14f
            setTextColor(Color.WHITE)
            setBackgroundColor(Color.parseColor("#0f3460"))
            setPadding(32, 16, 32, 16)
            layoutParams = LinearLayout.LayoutParams(0, LinearLayout.LayoutParams.WRAP_CONTENT, 1f).apply {
                setMargins(0, 0, 8, 0)
            }
            setOnClickListener {
                startService(Intent(this@MainActivity, PocketClawService::class.java))
                statusText.text = "Agent is running ✅"
                statusText.setTextColor(Color.parseColor("#00ff88"))
                startLogCapture()
            }
        }
        buttonRow.addView(startButton)

        val stopButton = Button(this).apply {
            text = "⏹ Stop"
            textSize = 14f
            setTextColor(Color.WHITE)
            setBackgroundColor(Color.parseColor("#e94560"))
            setPadding(32, 16, 32, 16)
            layoutParams = LinearLayout.LayoutParams(0, LinearLayout.LayoutParams.WRAP_CONTENT, 1f).apply {
                setMargins(8, 0, 8, 0)
            }
            setOnClickListener {
                val intent = Intent(this@MainActivity, PocketClawService::class.java)
                intent.action = "STOP"
                startService(intent)
                statusText.text = "Agent stopped"
                statusText.setTextColor(Color.parseColor("#aaaaaa"))
                stopLogCapture()
            }
        }
        buttonRow.addView(stopButton)

        val settingsButton = Button(this).apply {
            text = "⚙"
            textSize = 14f
            setTextColor(Color.parseColor("#aaaaaa"))
            setBackgroundColor(Color.parseColor("#16213e"))
            setPadding(24, 16, 24, 16)
            layoutParams = LinearLayout.LayoutParams(LinearLayout.LayoutParams.WRAP_CONTENT, LinearLayout.LayoutParams.WRAP_CONTENT).apply {
                setMargins(8, 0, 0, 0)
            }
            setOnClickListener {
                startActivity(Intent(this@MainActivity, SetupActivity::class.java))
            }
        }
        buttonRow.addView(settingsButton)

        topLayout.addView(buttonRow)
        rootLayout.addView(topLayout)

        // ─── Log Terminal ───
        val logHeader = LinearLayout(this).apply {
            orientation = LinearLayout.HORIZONTAL
            setPadding(48, 8, 48, 8)
            gravity = Gravity.CENTER_VERTICAL
        }

        logHeader.addView(TextView(this).apply {
            text = "📋 Logs"
            textSize = 16f
            setTextColor(Color.WHITE)
            typeface = Typeface.DEFAULT_BOLD
            layoutParams = LinearLayout.LayoutParams(0, LinearLayout.LayoutParams.WRAP_CONTENT, 1f)
        })

        val clearButton = Button(this).apply {
            text = "Clear"
            textSize = 12f
            setTextColor(Color.parseColor("#aaaaaa"))
            setBackgroundColor(Color.TRANSPARENT)
            setOnClickListener {
                logView.text = ""
            }
        }
        logHeader.addView(clearButton)

        rootLayout.addView(logHeader)

        logScroll = ScrollView(this).apply {
            setBackgroundColor(Color.parseColor("#0d1117"))
            setPadding(24, 16, 24, 16)
            layoutParams = LinearLayout.LayoutParams(
                LinearLayout.LayoutParams.MATCH_PARENT,
                0,
                1f
            )
        }

        logView = TextView(this).apply {
            text = "Waiting for agent to start...\n"
            textSize = 11f
            setTextColor(Color.parseColor("#30ff30"))
            typeface = Typeface.MONOSPACE
            setTextIsSelectable(true)
        }
        logScroll.addView(logView)
        rootLayout.addView(logScroll)

        setContentView(rootLayout)
    }

    private fun startLogCapture() {
        if (isLogging) return
        isLogging = true

        // Clear logcat buffer first
        try {
            Runtime.getRuntime().exec(arrayOf("logcat", "-c"))
        } catch (_: Exception) {}

        logThread = Thread {
            try {
                val process = Runtime.getRuntime().exec(
                    arrayOf("logcat", "-v", "time", "-s", "PocketClaw:*", "RustStdoutStderr:*")
                )
                val reader = BufferedReader(InputStreamReader(process.inputStream))
                val handler = Handler(Looper.getMainLooper())

                var line: String?
                while (isLogging) {
                    line = reader.readLine()
                    if (line != null) {
                        val displayLine = line.substringAfter("): ", line)
                        handler.post {
                            logView.append("$displayLine\n")
                            logScroll.post {
                                logScroll.fullScroll(ScrollView.FOCUS_DOWN)
                            }
                        }
                    }
                }
                process.destroy()
            } catch (e: Exception) {
                val handler = Handler(Looper.getMainLooper())
                handler.post {
                    logView.append("Log error: ${e.message}\n")
                }
            }
        }
        logThread?.start()
    }

    private fun stopLogCapture() {
        isLogging = false
        logThread?.interrupt()
    }

    override fun onDestroy() {
        super.onDestroy()
        stopLogCapture()
    }
}
