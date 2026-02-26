package com.example.mobileusetest.compose.ui

import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.background
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.gestures.detectTapGestures
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowBack
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.navigation.NavController

@OptIn(ExperimentalMaterial3Api::class, ExperimentalFoundationApi::class)
@Composable
fun ButtonsPage(navController: NavController) {
    var lastAction by remember { mutableStateOf("None") }
    var tapCount by remember { mutableIntStateOf(0) }
    var isEnabled by remember { mutableStateOf(true) }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Buttons & Taps") },
                navigationIcon = {
                    IconButton(onClick = { navController.popBackStack() }) {
                        Icon(Icons.Default.ArrowBack, contentDescription = "Back")
                    }
                },
                colors = TopAppBarDefaults.topAppBarColors(
                    containerColor = MaterialTheme.colorScheme.primaryContainer
                )
            )
        }
    ) { paddingValues ->
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(paddingValues)
                .padding(16.dp)
                .verticalScroll(rememberScrollState())
        ) {
            // Status Card
            Card(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(bottom = 24.dp)
            ) {
                Column(modifier = Modifier.padding(16.dp)) {
                    Text(
                        text = "Last Action: $lastAction",
                        fontSize = 18.sp,
                        modifier = Modifier.semantics {
                            contentDescription = "Last Action: $lastAction"
                        }
                    )
                    Spacer(modifier = Modifier.height(8.dp))
                    Text(
                        text = "Tap Count: $tapCount",
                        fontSize = 18.sp,
                        modifier = Modifier.semantics {
                            contentDescription = "Tap Count: $tapCount"
                        }
                    )
                }
            }

            // Tap Me Button
            ElevatedButton(
                onClick = {
                    lastAction = "Single Tap"
                    tapCount++
                },
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(bottom = 16.dp)
                    .semantics { contentDescription = "Tap Me Button" }
            ) {
                Text("Tap Me")
            }

            // Double Tap Area
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .background(Color(0xFFBBDEFB), RoundedCornerShape(8.dp))
                    .padding(24.dp)
                    .pointerInput(Unit) {
                        detectTapGestures(
                            onDoubleTap = {
                                lastAction = "Double Tap"
                                tapCount += 2
                            }
                        )
                    }
                    .semantics { contentDescription = "Double Tap Area" },
                contentAlignment = Alignment.Center
            ) {
                Text("Double Tap Area")
            }

            Spacer(modifier = Modifier.height(16.dp))

            // Long Press Area
            Box(
                modifier = Modifier
                    .fillMaxWidth()
                    .background(Color(0xFFC8E6C9), RoundedCornerShape(8.dp))
                    .padding(24.dp)
                    .combinedClickable(
                        onClick = { },
                        onLongClick = {
                            lastAction = "Long Press"
                            tapCount += 5
                        }
                    )
                    .semantics { contentDescription = "Long Press Area" },
                contentAlignment = Alignment.Center
            ) {
                Text("Long Press Area")
            }

            Spacer(modifier = Modifier.height(16.dp))

            // Enabled/Disabled Button Row
            Row(
                modifier = Modifier.fillMaxWidth(),
                verticalAlignment = Alignment.CenterVertically
            ) {
                ElevatedButton(
                    onClick = { lastAction = "Enabled Button Tapped" },
                    enabled = isEnabled,
                    modifier = Modifier
                        .weight(1f)
                        .semantics {
                            contentDescription = if (isEnabled) "Enabled Button" else "Disabled Button"
                        }
                ) {
                    Text(if (isEnabled) "Enabled Button" else "Disabled Button")
                }
                Spacer(modifier = Modifier.width(16.dp))
                Switch(
                    checked = isEnabled,
                    onCheckedChange = { isEnabled = it },
                    modifier = Modifier.semantics {
                        contentDescription = "Toggle Button Enable"
                    }
                )
            }

            Spacer(modifier = Modifier.height(16.dp))

            // Reset Button
            OutlinedButton(
                onClick = {
                    lastAction = "Reset"
                    tapCount = 0
                },
                modifier = Modifier
                    .fillMaxWidth()
                    .semantics { contentDescription = "Reset Counter" }
            ) {
                Text("Reset Counter")
            }
        }
    }
}
