package com.example.mobileusetest.compose.ui

import androidx.compose.foundation.layout.*
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.navigation.NavController

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun HomePage(navController: NavController) {
    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Mobile Use Test (Compose)") },
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
        ) {
            Text(
                text = "Test Pages",
                fontSize = 24.sp,
                fontWeight = FontWeight.Bold,
                modifier = Modifier
                    .padding(bottom = 16.dp)
                    .semantics { contentDescription = "Test Pages Header" }
            )

            NavButton(
                label = "Buttons & Taps",
                icon = Icons.Default.TouchApp,
                onClick = { navController.navigate("buttons") }
            )
            Spacer(modifier = Modifier.height(8.dp))

            NavButton(
                label = "Text Inputs",
                icon = Icons.Default.Keyboard,
                onClick = { navController.navigate("inputs") }
            )
            Spacer(modifier = Modifier.height(8.dp))

            NavButton(
                label = "Scrollable Lists",
                icon = Icons.Default.List,
                onClick = { navController.navigate("lists") }
            )
            Spacer(modifier = Modifier.height(8.dp))

            NavButton(
                label = "Form Controls",
                icon = Icons.Default.CheckBox,
                onClick = { navController.navigate("forms") }
            )
        }
    }
}

@Composable
private fun NavButton(
    label: String,
    icon: ImageVector,
    onClick: () -> Unit
) {
    ElevatedButton(
        onClick = onClick,
        modifier = Modifier
            .fillMaxWidth()
            .height(56.dp)
            .semantics { contentDescription = label }
    ) {
        Icon(
            imageVector = icon,
            contentDescription = null,
            modifier = Modifier.padding(end = 8.dp)
        )
        Text(label)
    }
}
