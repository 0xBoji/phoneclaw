package com.pocketclaw.app

import android.content.Intent
import android.os.Bundle
import android.widget.Button
import android.widget.TextView
import androidx.appcompat.app.AppCompatActivity

class MainActivity : AppCompatActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        
        // Setup basic layout programmatically to avoid complex XML for now
        val layout = androidx.constraintlayout.widget.ConstraintLayout(this).apply {
            layoutParams = androidx.constraintlayout.widget.ConstraintLayout.LayoutParams(
                androidx.constraintlayout.widget.ConstraintLayout.LayoutParams.MATCH_PARENT,
                androidx.constraintlayout.widget.ConstraintLayout.LayoutParams.MATCH_PARENT
            )
        }

        val statusText = TextView(this).apply {
            text = "PocketClaw Mobile 🦞"
            textSize = 24f
            id = android.view.View.generateViewId()
        }
        
        val startButton = Button(this).apply {
            text = "Start Agent"
            id = android.view.View.generateViewId()
            setOnClickListener {
                startService(Intent(this@MainActivity, PocketClawService::class.java))
                statusText.text = "Agent Started!"
            }
        }

        val stopButton = Button(this).apply {
            text = "Stop Agent"
            id = android.view.View.generateViewId()
            setOnClickListener {
                val intent = Intent(this@MainActivity, PocketClawService::class.java)
                intent.action = "STOP"
                startService(intent)
                statusText.text = "Agent Stopped"
            }
        }

        // Add views and constraints (omitted for brevity, just adding to view)
        // In a real app we would use XML layout
        val linearLayout = android.widget.LinearLayout(this).apply {
            orientation = android.widget.LinearLayout.VERTICAL
            gravity = android.view.Gravity.CENTER
            addView(statusText)
            addView(startButton)
            addView(stopButton)
        }
        
        setContentView(linearLayout)
    }
}
