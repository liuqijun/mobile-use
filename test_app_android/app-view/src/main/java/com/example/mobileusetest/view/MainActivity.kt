package com.example.mobileusetest.view

import android.content.Intent
import android.os.Bundle
import androidx.appcompat.app.AppCompatActivity
import com.google.android.material.button.MaterialButton

class MainActivity : AppCompatActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)

        supportActionBar?.title = "Mobile Use Test (View)"

        findViewById<MaterialButton>(R.id.buttonsButton).setOnClickListener {
            startActivity(Intent(this, ButtonsActivity::class.java))
        }

        findViewById<MaterialButton>(R.id.inputsButton).setOnClickListener {
            startActivity(Intent(this, InputsActivity::class.java))
        }

        findViewById<MaterialButton>(R.id.listsButton).setOnClickListener {
            startActivity(Intent(this, ListsActivity::class.java))
        }

        findViewById<MaterialButton>(R.id.formsButton).setOnClickListener {
            startActivity(Intent(this, FormsActivity::class.java))
        }
    }
}
