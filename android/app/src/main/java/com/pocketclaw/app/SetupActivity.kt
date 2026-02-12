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

        // Load existing config if any
        val existingConfig = loadExistingConfig()

        // ─── Title ───
        layout.addView(TextView(this).apply {
            text = "🦞 PocketClaw Setup"
            textSize = 28f
            setTextColor(Color.WHITE)
            typeface = Typeface.DEFAULT_BOLD
            gravity = Gravity.CENTER
            setPadding(0, 24, 0, 8)
        })

        layout.addView(TextView(this).apply {
            text = if (existingConfig != null) "Edit your configuration" else "Configure your AI provider to get started"
            textSize = 14f
            setTextColor(Color.parseColor("#aaaaaa"))
            gravity = Gravity.CENTER
            setPadding(0, 0, 0, 48)
        })

        // ─── Section: AI Provider (Required) ───
        layout.addView(createSectionHeader("🤖 AI Provider (Required)"))

        layout.addView(createLabel("Provider"))
        val providers = arrayOf("openai", "google", "anthropic", "openrouter", "groq")
        val providerSpinner = Spinner(this).apply {
            adapter = ArrayAdapter(
                this@SetupActivity,
                android.R.layout.simple_spinner_dropdown_item,
                providers
            )
            setBackgroundColor(Color.parseColor("#16213e"))
        }
        // Restore provider selection
        existingConfig?.optString("_provider")?.let { saved ->
            val idx = providers.indexOf(saved)
            if (idx >= 0) providerSpinner.setSelection(idx)
        }
        layout.addView(providerSpinner)
        layout.addView(createSpacer())

        layout.addView(createLabel("API Key"))
        val apiKeyInput = createInput("sk-xxxxxxxxxxxxxxx", InputType.TYPE_CLASS_TEXT or InputType.TYPE_TEXT_VARIATION_PASSWORD)
        existingConfig?.optString("_api_key")?.let { if (it.isNotEmpty()) apiKeyInput.setText(it) }
        layout.addView(apiKeyInput)
        layout.addView(createSpacer())

        layout.addView(createLabel("Model"))
        val modelInput = createInput("gpt-4o-mini")
        existingConfig?.optString("_model")?.let { if (it.isNotEmpty()) modelInput.setText(it) }
        layout.addView(modelInput)
        layout.addView(createSpacer())

        layout.addView(createLabel("System Prompt"))
        val promptInput = createInput("You are a helpful AI assistant.", InputType.TYPE_CLASS_TEXT or InputType.TYPE_TEXT_FLAG_MULTI_LINE).apply {
            minLines = 3
            gravity = Gravity.TOP or Gravity.START
        }
        existingConfig?.optString("_system_prompt")?.let { if (it.isNotEmpty()) promptInput.setText(it) }
        layout.addView(promptInput)
        layout.addView(createSpacer())

        // ─── Section: Telegram (Optional) ───
        layout.addView(createSectionHeader("📱 Telegram Bot (Optional)"))

        layout.addView(createLabel("Bot Token"))
        val telegramInput = createInput("123456:ABC-DEF1234...", InputType.TYPE_CLASS_TEXT or InputType.TYPE_TEXT_VARIATION_PASSWORD)
        existingConfig?.optString("_telegram_token")?.let { if (it.isNotEmpty()) telegramInput.setText(it) }
        layout.addView(telegramInput)
        layout.addView(createHint("Get from @BotFather on Telegram"))
        layout.addView(createSpacer())

        // ─── Section: Discord (Optional) ───
        layout.addView(createSectionHeader("💬 Discord Bot (Optional)"))

        layout.addView(createLabel("Bot Token"))
        val discordInput = createInput("MTk1Njc5...", InputType.TYPE_CLASS_TEXT or InputType.TYPE_TEXT_VARIATION_PASSWORD)
        existingConfig?.optString("_discord_token")?.let { if (it.isNotEmpty()) discordInput.setText(it) }
        layout.addView(discordInput)
        layout.addView(createHint("Get from Discord Developer Portal"))
        layout.addView(createSpacer())

        // ─── Section: Web Search (Optional) ───
        layout.addView(createSectionHeader("🔍 Web Search (Optional)"))

        layout.addView(createLabel("Brave Search API Key"))
        val braveKeyInput = createInput("BSA...", InputType.TYPE_CLASS_TEXT or InputType.TYPE_TEXT_VARIATION_PASSWORD)
        existingConfig?.optString("_brave_key")?.let { if (it.isNotEmpty()) braveKeyInput.setText(it) }
        layout.addView(braveKeyInput)
        layout.addView(createHint("Get from brave.com/search/api"))
        layout.addView(createSpacer())

        // ─── Section: Voice / Groq (Optional) ───
        layout.addView(createSectionHeader("🎙️ Voice (Groq - Optional)"))

        layout.addView(createLabel("Groq API Key"))
        val groqInput = createInput("gsk_...", InputType.TYPE_CLASS_TEXT or InputType.TYPE_TEXT_VARIATION_PASSWORD)
        existingConfig?.optString("_groq_key")?.let { if (it.isNotEmpty()) groqInput.setText(it) }
        layout.addView(groqInput)
        layout.addView(createHint("Required for voice features"))
        layout.addView(createSpacer())

        // ─── Section: Google Sheets Memory (Optional) ───
        layout.addView(createSectionHeader("📊 Google Sheets Memory (Optional)"))

        layout.addView(createLabel("Spreadsheet ID"))
        val sheetIdInput = createInput("1BxiMVs0XRA5nFMdKvBdBZjgmUUqptlbs74OgvE2upms")
        existingConfig?.optJSONObject("google_sheets")?.let { 
            sheetIdInput.setText(it.optString("spreadsheet_id", ""))
        }
        layout.addView(sheetIdInput)
        layout.addView(createHint("From docs.google.com/spreadsheets/d/..."))
        layout.addView(createSpacer())
        
        layout.addView(createLabel("Service Account JSON"))
        val serviceAccountInput = createInput("Paste contents of service-account.json", InputType.TYPE_CLASS_TEXT or InputType.TYPE_TEXT_FLAG_MULTI_LINE).apply {
            minLines = 3
            gravity = Gravity.TOP or Gravity.START
        }
        existingConfig?.optJSONObject("google_sheets")?.let { 
            serviceAccountInput.setText(it.optString("service_account_json", ""))
        }
        layout.addView(serviceAccountInput)
        layout.addView(createHint("Must have Editor access to the sheet"))
        layout.addView(createSpacer())

        // ─── Config File Path (Debug Info) ───
        val configFile = File(filesDir, ".pocketclaw/config.json")
        layout.addView(TextView(this).apply {
            text = "📄 Config: ${configFile.absolutePath}"
            textSize = 10f
            setTextColor(Color.parseColor("#444444"))
            setPadding(0, 16, 0, 16)
        })

        // ─── Save Button ───
        val saveButton = Button(this).apply {
            text = "💾  Save & Start"
            textSize = 18f
            setTextColor(Color.WHITE)
            setBackgroundColor(Color.parseColor("#e94560"))
            setPadding(32, 24, 32, 24)
            setOnClickListener {
                val provider = providerSpinner.selectedItem.toString()
                val apiKey = apiKeyInput.text.toString().trim()
                val model = modelInput.text.toString().trim()
                val prompt = promptInput.text.toString().trim()
                val telegramToken = telegramInput.text.toString().trim()
                val discordToken = discordInput.text.toString().trim()
                val braveKey = braveKeyInput.text.toString().trim()
                
                val sheetId = sheetIdInput.text.toString().trim()
                val serviceAccount = serviceAccountInput.text.toString().trim()
                val groqKey = groqInput.text.toString().trim()

                if (apiKey.isEmpty()) {
                    Toast.makeText(this@SetupActivity, "API Key is required!", Toast.LENGTH_SHORT).show()
                    return@setOnClickListener
                }
                if (model.isEmpty()) {
                    Toast.makeText(this@SetupActivity, "Model is required!", Toast.LENGTH_SHORT).show()
                    return@setOnClickListener
                }

                saveConfig(
                    provider = provider,
                    apiKey = apiKey,
                    model = model,
                    systemPrompt = prompt.ifEmpty { "You are a helpful AI assistant." },
                    telegramToken = telegramToken,
                    discordToken = discordToken,
                    braveKey = braveKey,
                    sheetId = sheetId,
                    serviceAccount = serviceAccount,
                    groqKey = groqKey
                )
                Toast.makeText(this@SetupActivity, "✅ Config saved!", Toast.LENGTH_SHORT).show()

                startActivity(Intent(this@SetupActivity, MainActivity::class.java))
                finish()
            }
        }
        layout.addView(saveButton)

        scrollView.addView(layout)
        setContentView(scrollView)
    }
    
    private fun loadExistingConfig(): JSONObject? {
        val configFile = File(filesDir, ".pocketclaw/config.json")
        if (!configFile.exists()) return null

        return try {
            val json = JSONObject(configFile.readText())
            val result = JSONObject()

            // Extract provider info
            // Extract provider info
            val providers = json.optJSONObject("providers")
            if (providers != null) {
                // Find main provider (openai, google, anthropic, or groq if it's the only one)
                val priorityList = listOf("openai", "google", "anthropic", "openrouter", "groq")
                var providerFound = false
                for (p in priorityList) {
                    if (providers.has(p)) {
                        result.put("_provider", p)
                        val pObj = providers.optJSONObject(p)
                        if (pObj != null) {
                            result.put("_api_key", pObj.optString("api_key", ""))
                            result.put("_model", pObj.optString("model", ""))
                        }
                        providerFound = true
                        break
                    }
                }
                
                if (!providerFound) {
                    val keys = providers.keys()
                    if (keys.hasNext()) {
                        val p = keys.next()
                        result.put("_provider", p)
                        val pObj = providers.optJSONObject(p)
                        result.put("_api_key", pObj?.optString("api_key", "") ?: "")
                        result.put("_model", pObj?.optString("model", "") ?: "")
                    }
                }

                // Explicitly extract Groq key for Voice section
                val groqObj = providers.optJSONObject("groq")
                if (groqObj != null) {
                    result.put("_groq_key", groqObj.optString("api_key", ""))
                }
            }

            // Extract agents config
            val agents = json.optJSONObject("agents")
            val defaultAgent = agents?.optJSONObject("default")
            if (defaultAgent != null) {
                result.put("_system_prompt", defaultAgent.optString("system_prompt", ""))
            }

            // Extract telegram
            val telegram = json.optJSONObject("telegram")
            if (telegram != null) {
                result.put("_telegram_token", telegram.optString("token", ""))
            }

            // Extract discord
            val discord = json.optJSONObject("discord")
            if (discord != null) {
                result.put("_discord_token", discord.optString("token", ""))
            }

            // Extract web/brave
            val web = json.optJSONObject("web")
            if (web != null) {
                result.put("_brave_key", web.optString("brave_key", ""))
            }
            
            // Extract Google Sheets
            val sheets = json.optJSONObject("google_sheets")
            if (sheets != null) {
                result.put("google_sheets", sheets)
            }

            result
        } catch (e: Exception) {
            null
        }
    }

    private fun saveConfig(
        provider: String,
        apiKey: String,
        model: String,
        systemPrompt: String,
        telegramToken: String,
        discordToken: String,
        braveKey: String,
        sheetId: String,
        serviceAccount: String,
        groqKey: String
    ) {
        val configDir = File(filesDir, ".pocketclaw")
        if (!configDir.exists()) configDir.mkdirs()

        val workspaceDir = File(filesDir, "workspace")
        if (!workspaceDir.exists()) workspaceDir.mkdirs()

        val providerObj = JSONObject().apply {
            put("api_key", apiKey)
            put("model", model)
            // Set api_base for providers that need custom endpoints
            when (provider) {
                "openrouter" -> put("api_base", "https://openrouter.ai/api/v1")
            }
        }

        val providersObj = JSONObject()
        providersObj.put(provider, providerObj)
        
        if (groqKey.isNotEmpty() && provider != "groq") {
            providersObj.put("groq", JSONObject().apply { 
                put("api_key", groqKey) 
            })
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

            if (telegramToken.isNotEmpty()) {
                put("telegram", JSONObject().apply {
                    put("token", telegramToken)
                })
            }

            if (discordToken.isNotEmpty()) {
                put("discord", JSONObject().apply {
                    put("token", discordToken)
                })
            }

            if (braveKey.isNotEmpty()) {
                put("web", JSONObject().apply {
                    put("brave_key", braveKey)
                })
            }
            
            if (sheetId.isNotEmpty() && serviceAccount.isNotEmpty()) {
                put("google_sheets", JSONObject().apply {
                    put("spreadsheet_id", sheetId)
                    put("service_account_json", serviceAccount)
                })
            }
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

    // Helper functions
    private fun createLabel(text: String): TextView {
        return TextView(this).apply {
            this.text = text
            textSize = 14f
            setTextColor(Color.parseColor("#dddddd"))
            setPadding(0, 8, 0, 8)
        }
    }

    private fun createInput(hint: String, inputType: Int = InputType.TYPE_CLASS_TEXT): EditText {
        return EditText(this).apply {
            this.hint = hint
            this.inputType = inputType
            textSize = 14f
            setTextColor(Color.WHITE)
            setHintTextColor(Color.parseColor("#666666"))
            setBackgroundColor(Color.parseColor("#16213e"))
            setPadding(24, 24, 24, 24)
        }
    }

    private fun createSpacer(): View {
        return View(this).apply {
            layoutParams = LinearLayout.LayoutParams(LinearLayout.LayoutParams.MATCH_PARENT, 32)
        }
    }
    
    private fun createHint(text: String): TextView {
        return TextView(this).apply {
            this.text = text
            textSize = 12f
            setTextColor(Color.parseColor("#888888"))
            setPadding(0, 4, 0, 16)
        }
    }

    private fun createSectionHeader(title: String): TextView {
        return TextView(this).apply {
            text = title
            textSize = 18f
            setTextColor(Color.parseColor("#e94560"))
            typeface = Typeface.DEFAULT_BOLD
            setPadding(0, 32, 0, 16)
        }
    }
}
