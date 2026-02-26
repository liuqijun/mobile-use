package com.example.mobileusetest.view

import android.os.Bundle
import android.view.GestureDetector
import android.view.MotionEvent
import android.widget.FrameLayout
import android.widget.TextView
import androidx.appcompat.app.AppCompatActivity
import androidx.core.view.GestureDetectorCompat
import com.google.android.material.button.MaterialButton
import com.google.android.material.switchmaterial.SwitchMaterial

class ButtonsActivity : AppCompatActivity() {
    private var lastAction = "None"
    private var tapCount = 0
    private var isEnabled = true

    private lateinit var lastActionText: TextView
    private lateinit var tapCountText: TextView
    private lateinit var toggleableButton: MaterialButton

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_buttons)

        supportActionBar?.title = "Buttons & Taps"
        supportActionBar?.setDisplayHomeAsUpEnabled(true)

        lastActionText = findViewById(R.id.lastActionText)
        tapCountText = findViewById(R.id.tapCountText)
        toggleableButton = findViewById(R.id.toggleableButton)

        // Tap Me Button
        findViewById<MaterialButton>(R.id.tapMeButton).setOnClickListener {
            lastAction = "Single Tap"
            tapCount++
            updateStatus()
        }

        // Double Tap Area
        val doubleTapArea = findViewById<FrameLayout>(R.id.doubleTapArea)
        val doubleTapDetector = GestureDetectorCompat(this, object : GestureDetector.SimpleOnGestureListener() {
            override fun onDoubleTap(e: MotionEvent): Boolean {
                lastAction = "Double Tap"
                tapCount += 2
                updateStatus()
                return true
            }
        })
        doubleTapArea.setOnTouchListener { _, event ->
            doubleTapDetector.onTouchEvent(event)
            true
        }

        // Long Press Area
        findViewById<FrameLayout>(R.id.longPressArea).setOnLongClickListener {
            lastAction = "Long Press"
            tapCount += 5
            updateStatus()
            true
        }

        // Enable Switch
        findViewById<SwitchMaterial>(R.id.enableSwitch).setOnCheckedChangeListener { _, checked ->
            isEnabled = checked
            toggleableButton.isEnabled = checked
            toggleableButton.text = if (checked) "Enabled Button" else "Disabled Button"
            toggleableButton.contentDescription = if (checked) "Enabled Button" else "Disabled Button"
        }

        // Toggleable Button
        toggleableButton.setOnClickListener {
            lastAction = "Enabled Button Tapped"
            updateStatus()
        }

        // Reset Button
        findViewById<MaterialButton>(R.id.resetButton).setOnClickListener {
            lastAction = "Reset"
            tapCount = 0
            updateStatus()
        }
    }

    private fun updateStatus() {
        lastActionText.text = "Last Action: $lastAction"
        lastActionText.contentDescription = "Last Action: $lastAction"
        tapCountText.text = "Tap Count: $tapCount"
        tapCountText.contentDescription = "Tap Count: $tapCount"
    }

    override fun onSupportNavigateUp(): Boolean {
        finish()
        return true
    }
}
