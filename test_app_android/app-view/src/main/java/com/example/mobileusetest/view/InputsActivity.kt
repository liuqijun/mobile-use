package com.example.mobileusetest.view

import android.os.Bundle
import android.view.View
import android.widget.TextView
import androidx.appcompat.app.AppCompatActivity
import com.google.android.material.button.MaterialButton
import com.google.android.material.card.MaterialCardView
import com.google.android.material.textfield.TextInputEditText

class InputsActivity : AppCompatActivity() {
    private lateinit var usernameInput: TextInputEditText
    private lateinit var passwordInput: TextInputEditText
    private lateinit var emailInput: TextInputEditText
    private lateinit var searchInput: TextInputEditText
    private lateinit var resultCard: MaterialCardView
    private lateinit var resultText: TextView

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_inputs)

        supportActionBar?.title = "Text Inputs"
        supportActionBar?.setDisplayHomeAsUpEnabled(true)

        usernameInput = findViewById(R.id.usernameInput)
        passwordInput = findViewById(R.id.passwordInput)
        emailInput = findViewById(R.id.emailInput)
        searchInput = findViewById(R.id.searchInput)
        resultCard = findViewById(R.id.resultCard)
        resultText = findViewById(R.id.resultText)

        findViewById<MaterialButton>(R.id.submitButton).setOnClickListener {
            val result = """
                Username: ${usernameInput.text}
                Email: ${emailInput.text}
                Search: ${searchInput.text}
            """.trimIndent()
            resultText.text = result
            resultText.contentDescription = "Submitted Data: $result"
            resultCard.visibility = View.VISIBLE
        }

        findViewById<MaterialButton>(R.id.clearAllButton).setOnClickListener {
            usernameInput.text?.clear()
            passwordInput.text?.clear()
            emailInput.text?.clear()
            searchInput.text?.clear()
            resultText.text = "All fields cleared"
            resultText.contentDescription = "Submitted Data: All fields cleared"
            resultCard.visibility = View.VISIBLE
        }
    }

    override fun onSupportNavigateUp(): Boolean {
        finish()
        return true
    }
}
