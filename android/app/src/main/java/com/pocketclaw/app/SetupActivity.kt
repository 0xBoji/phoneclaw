package com.pocketclaw.app

import android.content.Context
import android.content.Intent
import android.graphics.Color
import android.graphics.Typeface
import android.os.Bundle
import android.text.InputType
import android.view.Gravity
import android.view.View
import android.widget.*
import androidx.appcompat.app.AppCompatActivity
import org.json.JSONObject
import java.io.File

class SetupActivity : AppCompatActivity() {

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        val scrollView = ScrollView(this).apply {
            setBackgroundColor(Color.parseColor("#1a1a2e"))
            setPadding(48, 48, 48, 48)
        }

        val layout = LinearLayout(this).apply {
            orientation = LinearLayout.VERTICAL
            gravity = Gravity.CENTER_HORIZONTAL
        }

        // Title
        layout.addView(TextView(this).apply {
            text = "🦞 PocketClaw Setup"
            textSize = 28f
            setTextColor(Color.WHITE)
            typeface = Typeface.DEFAULT_BOLD
            gravity = Gravity.CENTER
            setPadding(0, 24, 0, 8)
        })

        layout.addView(TextView(this).apply {
            text = "Configure your AI provider to get started"
            textSize = 14f
            setTextColor(Color.parseColor("#aaaaaa"))
            gravity = Gravity.CENTER
            setPadding(0, 0, 0, 48)
        })

        // Provider Selector
        layout.addView(createLabel("Provider"))
        val providerSpinner = Spinner(this).apply {
            adapter = ArrayAdapter(
                this@SetupActivity,
                android.R.layout.simple_spinner_dropdown_item,
                arrayOf("openai", "google", "anthropic", "openrouter", "groq")
            )
            setBackgroundColor(Color.parseColor("#16213e"))
        }
        layout.addView(providerSpinner)
        layout.addView(createSpacer())

        // API Key
        layout.addView(createLabel("API Key"))
        val apiKeyInput = createInput("sk-xxxxxxxxxxxxxxx", InputType.TYPE_CLASS_TEXT or InputType.TYPE_TEXT_VARIATION_PASSWORD)
        layout.addView(apiKeyInput)
        layout.addView(createSpacer())

        // Model
        layout.addView(createLabel("Model"))
        val modelInput = createInput("gpt-4o-mini")
        layout.addView(modelInput)
        layout.addView(createSpacer())

        // System Prompt
        layout.addView(createLabel("System Prompt (optional)"))
        val promptInput = createInput("You are a helpful AI assistant.", InputType.TYPE_CLASS_TEXT or InputType.TYPE_TEXT_FLAG_MULTI_LINE).apply {
            minLines = 3
            gravity = Gravity.TOP or Gravity.START
        }
        layout.addView(promptInput)
        layout.addView(createSpacer())

        // Save Button
        val saveButton = Button(this).apply {
            text = "Save & Start"
            textSize = 18f
            setTextColor(Color.WHITE)
            setBackgroundColor(Color.parseColor("#e94560"))
            setPadding(32, 24, 32, 24)
            setOnClickListener {
                val provider = providerSpinner.selectedItem.toString()
                val apiKey = apiKeyInput.text.toString().trim()
                val model = modelInput.text.toString().trim()
                val prompt = promptInput.text.toString().trim()

                if (apiKey.isEmpty()) {
                    Toast.makeText(this@SetupActivity, "API Key is required!", Toast.LENGTH_SHORT).show()
                    return@setOnClickListener
                }
                if (model.isEmpty()) {
                    Toast.makeText(this@SetupActivity, "Model is required!", Toast.LENGTH_SHORT).show()
                    return@setOnClickListener
                }

                saveConfig(provider, apiKey, model, prompt.ifEmpty { "You are a helpful AI assistant." })
                Toast.makeText(this@SetupActivity, "Config saved! Starting agent...", Toast.LENGTH_SHORT).show()

                startActivity(Intent(this@SetupActivity, MainActivity::class.java))
                finish()
            }
        }
        layout.addView(saveButton)

        scrollView.addView(layout)
        setContentView(scrollView)
    }

    private fun createLabel(text: String): TextView {
        return TextView(this).apply {
            this.text = text
            textSize = 16f
            setTextColor(Color.parseColor("#e94560"))
            typeface = Typeface.DEFAULT_BOLD
            setPadding(0, 0, 0, 8)
        }
    }

    private fun createInput(hint: String, type: Int = InputType.TYPE_CLASS_TEXT): EditText {
        return EditText(this).apply {
            this.hint = hint
            this.inputType = type
            textSize = 16f
            setTextColor(Color.WHITE)
            setHintTextColor(Color.parseColor("#666666"))
            setBackgroundColor(Color.parseColor("#16213e"))
            setPadding(24, 20, 24, 20)
            layoutParams = LinearLayout.LayoutParams(
                LinearLayout.LayoutParams.MATCH_PARENT,
                LinearLayout.LayoutParams.WRAP_CONTENT
            )
        }
    }

    private fun createSpacer(): View {
        return View(this).apply {
            layoutParams = LinearLayout.LayoutParams(
                LinearLayout.LayoutParams.MATCH_PARENT, 24
            )
        }
    }

    private fun saveConfig(provider: String, apiKey: String, model: String, systemPrompt: String) {
        val configDir = File(filesDir, ".pocketclaw")
        if (!configDir.exists()) configDir.mkdirs()

        val workspaceDir = File(filesDir, "workspace")
        if (!workspaceDir.exists()) workspaceDir.mkdirs()

        val providerObj = JSONObject().apply {
            put("api_key", apiKey)
            put("model", model)
        }

        val providersObj = JSONObject().apply {
            put(provider, providerObj)
        }

        val agentsObj = JSONObject().apply {
            put("default", JSONObject().apply {
                put("model", model)
                put("system_prompt", systemPrompt)
                put("max_tokens", 4096)
                put("temperature", 0.7)
            })
        }

        val config = JSONObject().apply {
            put("workspace", workspaceDir.absolutePath)
            put("providers", providersObj)
            put("agents", agentsObj)
        }

        val configFile = File(configDir, "config.json")
        configFile.writeText(config.toString(2))
    }

    companion object {
        fun hasConfig(context: Context): Boolean {
            val configFile = File(context.filesDir, ".pocketclaw/config.json")
            return configFile.exists()
        }
    }
}
