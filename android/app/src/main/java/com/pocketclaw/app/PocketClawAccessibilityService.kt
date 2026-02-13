package com.pocketclaw.app

import android.accessibilityservice.AccessibilityService
import android.accessibilityservice.GestureDescription
import android.graphics.Path
import android.view.accessibility.AccessibilityEvent
import android.view.accessibility.AccessibilityNodeInfo
import android.util.Log

class PocketClawAccessibilityService : AccessibilityService() {

    companion object {
        var instance: PocketClawAccessibilityService? = null
    }

    override fun onServiceConnected() {
        super.onServiceConnected()
        instance = this
        Log.d("PocketClaw", "Accessibility Service Connected")
    }

    override fun onAccessibilityEvent(event: AccessibilityEvent?) {
        // We can listen to events here if needed, e.g., window state changes
    }

    override fun onInterrupt() {
        Log.d("PocketClaw", "Accessibility Service Interrupted")
    }

    override fun onDestroy() {
        super.onDestroy()
        instance = null
    }

    // --- Action Methods ---

    fun click(x: Float, y: Float): Boolean {
        val path = Path()
        path.moveTo(x, y)
        val builder = GestureDescription.Builder()
        val gesture = builder.addStroke(GestureDescription.StrokeDescription(path, 0, 50))
            .build()
        return dispatchGesture(gesture, null, null)
    }

    fun swipe(x1: Float, y1: Float, x2: Float, y2: Float, duration: Long = 300): Boolean {
        val path = Path()
        path.moveTo(x1, y1)
        path.lineTo(x2, y2)
        val builder = GestureDescription.Builder()
        val gesture = builder.addStroke(GestureDescription.StrokeDescription(path, 0, duration))
            .build()
        return dispatchGesture(gesture, null, null)
    }

    fun back(): Boolean {
        return performGlobalAction(GLOBAL_ACTION_BACK)
    }

    fun home(): Boolean {
        return performGlobalAction(GLOBAL_ACTION_HOME)
    }

    fun recentApps(): Boolean {
        return performGlobalAction(GLOBAL_ACTION_RECENTS)
    }

    // Improved node finding and clicking
    // This finds a node by text (case-insensitive) and clicks it
    fun clickNodeByText(text: String): Boolean {
        val root = rootInActiveWindow ?: return false
        val nodes = root.findAccessibilityNodeInfosByText(text)
        for (node in nodes) {
            // Check if the node is clickable, or find a clickable parent
            var clickableNode = node
            while (clickableNode != null) {
                if (clickableNode.isClickable) {
                    val result = clickableNode.performAction(AccessibilityNodeInfo.ACTION_CLICK)
                    clickableNode.recycle()
                    return result
                }
                clickableNode = clickableNode.parent
            }
            node.recycle()
        }
        return false
    }
}
