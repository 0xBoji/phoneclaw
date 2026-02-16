package com.phoneclaw.app.wave

import android.content.Context
import android.graphics.Color
import android.graphics.Typeface
import android.graphics.drawable.GradientDrawable
import android.text.InputType
import android.view.Gravity
import android.view.View
import android.widget.Button
import android.widget.EditText
import android.widget.LinearLayout
import android.widget.ScrollView
import android.widget.TextView

object UiFactory {
    private const val COLOR_BG = "#1F1954"
    private const val COLOR_CARD = "#33FFFFFF"
    private const val COLOR_CARD_ALT = "#1FFFFFFF"
    private const val COLOR_TEXT_PRIMARY = "#F3EEFF"
    private const val COLOR_TEXT_SECONDARY = "#CDC2F5"
    private const val COLOR_TEXT_MUTED = "#AA9ED8"
    private const val COLOR_BORDER = "#66F5E6FF"
    private const val COLOR_PRIMARY = "#9B4DFF"
    private const val COLOR_PRIMARY_DARK = "#7B2FF7"
    private const val COLOR_PRIMARY_SOFT = "#2EE6A8FF"

    fun colorTextPrimary(): Int = Color.parseColor(COLOR_TEXT_PRIMARY)
    fun colorTextSecondary(): Int = Color.parseColor(COLOR_TEXT_SECONDARY)
    fun colorPrimaryDark(): Int = Color.parseColor(COLOR_PRIMARY_DARK)
    fun colorPrimarySoft(): Int = Color.parseColor(COLOR_PRIMARY_SOFT)
    fun colorBorder(): Int = Color.parseColor(COLOR_BORDER)

    private fun roundedBackground(fill: String, stroke: String? = null, radius: Float = 22f): GradientDrawable {
        return GradientDrawable().apply {
            shape = GradientDrawable.RECTANGLE
            cornerRadius = radius
            setColor(Color.parseColor(fill))
            if (stroke != null) {
                setStroke(2, Color.parseColor(stroke))
            }
        }
    }

    private fun neonActionBackground(): GradientDrawable {
        return GradientDrawable(
            GradientDrawable.Orientation.LEFT_RIGHT,
            intArrayOf(
                Color.parseColor("#FF4D9E"),
                Color.parseColor("#FF6A4D"),
            )
        ).apply {
            shape = GradientDrawable.RECTANGLE
            cornerRadius = 14f
            setStroke(2, Color.parseColor("#66FFFFFF"))
        }
    }

    fun screen(context: Context): Pair<ScrollView, LinearLayout> {
        val scroll = ScrollView(context).apply {
            setBackgroundColor(Color.parseColor(COLOR_BG))
            setPadding(12, 12, 12, 12)
        }
        val root = LinearLayout(context).apply {
            orientation = LinearLayout.VERTICAL
            layoutParams = LinearLayout.LayoutParams(
                LinearLayout.LayoutParams.MATCH_PARENT,
                LinearLayout.LayoutParams.WRAP_CONTENT
            )
            background = roundedBackground(COLOR_CARD, COLOR_BORDER, 22f)
            elevation = 7f
            setPadding(14, 14, 14, 14)
        }
        scroll.addView(root)
        return Pair(scroll, root)
    }

    fun title(context: Context, text: String): TextView = TextView(context).apply {
        this.text = text
        textSize = 21f
        setTextColor(colorTextPrimary())
        typeface = Typeface.DEFAULT_BOLD
        setPadding(0, 0, 0, 6)
    }

    fun subtitle(context: Context, text: String): TextView = TextView(context).apply {
        this.text = text
        textSize = 12f
        setTextColor(colorTextSecondary())
        setPadding(0, 0, 0, 12)
    }

    fun section(context: Context, text: String): TextView = TextView(context).apply {
        this.text = text
        textSize = 15f
        setTextColor(colorPrimaryDark())
        typeface = Typeface.DEFAULT_BOLD
        setPadding(0, 10, 0, 7)
    }

    fun label(context: Context, text: String): TextView = TextView(context).apply {
        this.text = text
        textSize = 12f
        setTextColor(colorTextPrimary())
        setPadding(0, 6, 0, 4)
    }

    fun hint(context: Context, text: String): TextView = TextView(context).apply {
        this.text = text
        textSize = 10f
        setTextColor(Color.parseColor(COLOR_TEXT_MUTED))
        setPadding(0, 2, 0, 8)
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
            textSize = 12f
            setTextColor(colorTextPrimary())
            setHintTextColor(Color.parseColor(COLOR_TEXT_MUTED))
            background = roundedBackground(COLOR_CARD_ALT, COLOR_BORDER, 14f)
            setPadding(18, 14, 18, 14)
            if (multiline) {
                minLines = 3
                gravity = Gravity.TOP or Gravity.START
            }
        }
    }

    fun spacer(context: Context, h: Int = 8): View = View(context).apply {
        layoutParams = LinearLayout.LayoutParams(LinearLayout.LayoutParams.MATCH_PARENT, h)
    }

    fun actionButton(context: Context, text: String): Button = Button(context).apply {
        this.text = text
        textSize = 13f
        isAllCaps = false
        setTextColor(Color.WHITE)
        background = neonActionBackground()
        setPadding(16, 14, 16, 14)
        minHeight = 0
    }

    fun chipButton(context: Context, text: String): Button = Button(context).apply {
        this.text = text
        textSize = 11f
        isAllCaps = false
        setTextColor(colorTextPrimary())
        background = roundedBackground(COLOR_CARD_ALT, COLOR_BORDER, 999f)
        setPadding(14, 10, 14, 10)
        minHeight = 0
    }

    fun secondaryButton(context: Context, text: String): Button = Button(context).apply {
        this.text = text
        textSize = 12f
        isAllCaps = false
        setTextColor(colorTextPrimary())
        background = roundedBackground(COLOR_CARD_ALT, COLOR_BORDER, 14f)
        setPadding(16, 12, 16, 12)
        minHeight = 0
    }

    fun surfaceCard(context: Context): LinearLayout = LinearLayout(context).apply {
        orientation = LinearLayout.VERTICAL
        background = roundedBackground(COLOR_CARD_ALT, COLOR_BORDER, 14f)
        setPadding(12, 12, 12, 12)
        elevation = 1f
    }
}
