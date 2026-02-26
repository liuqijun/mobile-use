package com.example.mobileusetest.view

import android.os.Bundle
import android.widget.CheckBox
import android.widget.RadioGroup
import android.widget.TextView
import androidx.appcompat.app.AppCompatActivity
import com.google.android.material.slider.Slider
import com.google.android.material.switchmaterial.SwitchMaterial

class FormsActivity : AppCompatActivity() {
    private lateinit var checkbox1: CheckBox
    private lateinit var checkbox2: CheckBox
    private lateinit var switch1: SwitchMaterial
    private lateinit var switch2: SwitchMaterial
    private lateinit var slider: Slider
    private lateinit var sliderValue: TextView
    private lateinit var radioGroup: RadioGroup
    private lateinit var stateText: TextView

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_forms)

        supportActionBar?.title = "Form Controls"
        supportActionBar?.setDisplayHomeAsUpEnabled(true)

        checkbox1 = findViewById(R.id.checkbox1)
        checkbox2 = findViewById(R.id.checkbox2)
        switch1 = findViewById(R.id.switch1)
        switch2 = findViewById(R.id.switch2)
        slider = findViewById(R.id.slider)
        sliderValue = findViewById(R.id.sliderValue)
        radioGroup = findViewById(R.id.radioGroup)
        stateText = findViewById(R.id.stateText)

        // Set up listeners
        checkbox1.setOnCheckedChangeListener { _, _ -> updateState() }
        checkbox2.setOnCheckedChangeListener { _, _ -> updateState() }
        switch1.setOnCheckedChangeListener { _, _ -> updateState() }
        switch2.setOnCheckedChangeListener { _, _ -> updateState() }

        slider.addOnChangeListener { _, value, _ ->
            val intValue = value.toInt()
            sliderValue.text = "Value: $intValue"
            sliderValue.contentDescription = "Slider Value: $intValue"
            updateState()
        }

        radioGroup.setOnCheckedChangeListener { _, _ -> updateState() }

        updateState()
    }

    private fun updateState() {
        val radioValue = when (radioGroup.checkedRadioButtonId) {
            R.id.radioA -> 1
            R.id.radioB -> 2
            R.id.radioC -> 3
            else -> 0
        }

        val state = """
            Checkbox 1: ${checkbox1.isChecked}
            Checkbox 2: ${checkbox2.isChecked}
            Switch 1: ${switch1.isChecked}
            Switch 2: ${switch2.isChecked}
            Slider: ${slider.value.toInt()}
            Radio: $radioValue
        """.trimIndent()

        stateText.text = state
        stateText.contentDescription = "Current State: $state"
    }

    override fun onSupportNavigateUp(): Boolean {
        finish()
        return true
    }
}
