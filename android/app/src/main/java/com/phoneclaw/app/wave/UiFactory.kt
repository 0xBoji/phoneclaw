package com.phoneclaw.app.wave

import com.phoneclaw.app.R
import android.content.Context
import android.graphics.Color
import android.graphics.Typeface
import android.text.InputType
import android.view.Gravity
import android.view.View
import android.widget.Button
import android.widget.EditText
import android.widget.LinearLayout
import android.widget.ScrollView
import android.widget.TextView

object UiFactory {
    private const val COLOR_TEXT_PRIMARY = "#24323D"
    private const val COLOR_TEXT_SECONDARY = "#4C6674"
    private const val COLOR_TEXT_MUTED = "#6F8793"
    private const val COLOR_SURFACE = "#F5F8FB"
    private const val COLOR_SURFACE_ALT = "#EAF2F7"
    private const val COLOR_PRIMARY = "#6C9EC1"
    private const val COLOR_PRIMARY_DARK = "#5B87A7"

    fun screen(context: Context): Pair<ScrollView, LinearLayout> {
        val scroll = ScrollView(context).apply {
            setBackgroundResource(R.drawable.pastel)
            setPadding(36, 36, 36, 36)
        }
        val root = LinearLayout(context).apply {
            orientation = LinearLayout.VERTICAL
            gravity = Gravity.CENTER_HORIZONTAL
            setBackgroundColor(Color.parseColor("#CCFFFFFF"))
            setPadding(24, 24, 24, 24)
        }
        scroll.addView(root)
        return Pair(scroll, root)
    }

    fun title(context: Context, text: String): TextView = TextView(context).apply {
        this.text = text
        textSize = 24f
        setTextColor(Color.parseColor(COLOR_TEXT_PRIMARY))
        typeface = Typeface.DEFAULT_BOLD
        setPadding(0, 16, 0, 8)
    }

    fun subtitle(context: Context, text: String): TextView = TextView(context).apply {
        this.text = text
        textSize = 13f
        setTextColor(Color.parseColor(COLOR_TEXT_SECONDARY))
        setPadding(0, 0, 0, 28)
    }

    fun section(context: Context, text: String): TextView = TextView(context).apply {
        this.text = text
        textSize = 17f
        setTextColor(Color.parseColor(COLOR_PRIMARY_DARK))
        typeface = Typeface.DEFAULT_BOLD
        setPadding(0, 20, 0, 10)
    }

    fun label(context: Context, text: String): TextView = TextView(context).apply {
        this.text = text
        textSize = 13f
        setTextColor(Color.parseColor(COLOR_TEXT_PRIMARY))
        setPadding(0, 8, 0, 6)
    }

    fun hint(context: Context, text: String): TextView = TextView(context).apply {
        this.text = text
        textSize = 11f
        setTextColor(Color.parseColor(COLOR_TEXT_MUTED))
        setPadding(0, 4, 0, 14)
    }

    fun input(context: Context, hint: String, multiline: Boolean = false, secret: Boolean = false): EditText {
        val inputType = when {
            secret -> InputType.TYPE_CLASS_TEXT or InputType.TYPE_TEXT_VARIATION_PASSWORD
            multiline -> InputType.TYPE_CLASS_TEXT or InputType.TYPE_TEXT_FLAG_MULTI_LINE
            else -> InputType.TYPE_CLASS_TEXT
        }

        return EditText(context).apply {
            this.hint = hint
            this.inputType = inputType
            textSize = 14f
            setTextColor(Color.parseColor(COLOR_TEXT_PRIMARY))
            setHintTextColor(Color.parseColor(COLOR_TEXT_MUTED))
            setBackgroundColor(Color.parseColor(COLOR_SURFACE))
            setPadding(20, 20, 20, 20)
            if (multiline) {
                minLines = 3
                gravity = Gravity.TOP or Gravity.START
            }
        }
    }

    fun spacer(context: Context, h: Int = 18): View = View(context).apply {
        layoutParams = LinearLayout.LayoutParams(LinearLayout.LayoutParams.MATCH_PARENT, h)
    }

    fun actionButton(context: Context, text: String): Button = Button(context).apply {
        this.text = text
        textSize = 15f
        setTextColor(Color.WHITE)
        setBackgroundColor(Color.parseColor(COLOR_PRIMARY))
        setPadding(20, 22, 20, 22)
    }

    fun secondaryButton(context: Context, text: String): Button = Button(context).apply {
        this.text = text
        textSize = 14f
        setTextColor(Color.parseColor(COLOR_TEXT_PRIMARY))
        setBackgroundColor(Color.parseColor(COLOR_SURFACE_ALT))
        setPadding(20, 18, 20, 18)
    }
}
