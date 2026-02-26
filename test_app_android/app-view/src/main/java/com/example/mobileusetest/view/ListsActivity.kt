package com.example.mobileusetest.view

import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.widget.TextView
import androidx.appcompat.app.AppCompatActivity
import androidx.recyclerview.widget.LinearLayoutManager
import androidx.recyclerview.widget.RecyclerView
import com.google.android.material.snackbar.Snackbar

class ListsActivity : AppCompatActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_lists)

        supportActionBar?.title = "Scrollable Lists"
        supportActionBar?.setDisplayHomeAsUpEnabled(true)

        val recyclerView = findViewById<RecyclerView>(R.id.recyclerView)
        recyclerView.layoutManager = LinearLayoutManager(this)
        recyclerView.adapter = ListAdapter(50) { index ->
            Snackbar.make(recyclerView, "Tapped item ${index + 1}", Snackbar.LENGTH_SHORT).show()
        }
    }

    override fun onSupportNavigateUp(): Boolean {
        finish()
        return true
    }

    private class ListAdapter(
        private val itemCount: Int,
        private val onItemClick: (Int) -> Unit
    ) : RecyclerView.Adapter<ListAdapter.ViewHolder>() {

        class ViewHolder(view: View) : RecyclerView.ViewHolder(view) {
            val avatarText: TextView = view.findViewById(R.id.avatarText)
            val titleText: TextView = view.findViewById(R.id.titleText)
            val subtitleText: TextView = view.findViewById(R.id.subtitleText)
        }

        override fun onCreateViewHolder(parent: ViewGroup, viewType: Int): ViewHolder {
            val view = LayoutInflater.from(parent.context)
                .inflate(R.layout.item_list, parent, false)
            return ViewHolder(view)
        }

        override fun onBindViewHolder(holder: ViewHolder, position: Int) {
            val index = position + 1
            holder.avatarText.text = index.toString()
            holder.titleText.text = "List Item $index"
            holder.subtitleText.text = "Description for item $index"
            holder.itemView.contentDescription = "List Item $index"
            holder.itemView.setOnClickListener { onItemClick(position) }
        }

        override fun getItemCount() = itemCount
    }
}
