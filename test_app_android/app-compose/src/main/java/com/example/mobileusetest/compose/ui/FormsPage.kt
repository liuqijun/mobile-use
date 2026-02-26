package com.example.mobileusetest.compose.ui

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.selection.selectable
import androidx.compose.foundation.selection.selectableGroup
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowBack
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.navigation.NavController

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun FormsPage(navController: NavController) {
    var checkbox1 by remember { mutableStateOf(false) }
    var checkbox2 by remember { mutableStateOf(true) }
    var switch1 by remember { mutableStateOf(false) }
    var switch2 by remember { mutableStateOf(true) }
    var sliderValue by remember { mutableFloatStateOf(50f) }
    var radioValue by remember { mutableIntStateOf(1) }

    Scaffold(
        topBar = {
            TopAppBar(
                title = { Text("Form Controls") },
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
            // Checkboxes Section
            Text(
                text = "Checkboxes",
                fontSize = 18.sp,
                fontWeight = FontWeight.Bold,
                modifier = Modifier.padding(bottom = 8.dp)
            )

            LabeledCheckbox(
                checked = checkbox1,
                onCheckedChange = { checkbox1 = it },
                label = "Option 1 (Unchecked by default)",
                description = "Option 1"
            )

            LabeledCheckbox(
                checked = checkbox2,
                onCheckedChange = { checkbox2 = it },
                label = "Option 2 (Checked by default)",
                description = "Option 2"
            )

            LabeledCheckbox(
                checked = false,
                onCheckedChange = null,
                label = "Option 3 (Disabled)",
                description = "Option 3 (Disabled)",
                enabled = false
            )

            Divider(modifier = Modifier.padding(vertical = 16.dp))

            // Switches Section
            Text(
                text = "Switches",
                fontSize = 18.sp,
                fontWeight = FontWeight.Bold,
                modifier = Modifier.padding(bottom = 8.dp)
            )

            LabeledSwitch(
                checked = switch1,
                onCheckedChange = { switch1 = it },
                label = "Toggle 1 (Off by default)",
                description = "Toggle 1"
            )

            LabeledSwitch(
                checked = switch2,
                onCheckedChange = { switch2 = it },
                label = "Toggle 2 (On by default)",
                description = "Toggle 2"
            )

            Divider(modifier = Modifier.padding(vertical = 16.dp))

            // Slider Section
            Text(
                text = "Slider",
                fontSize = 18.sp,
                fontWeight = FontWeight.Bold,
                modifier = Modifier.padding(bottom = 8.dp)
            )

            Slider(
                value = sliderValue,
                onValueChange = { sliderValue = it },
                valueRange = 0f..100f,
                steps = 9,
                modifier = Modifier
                    .fillMaxWidth()
                    .semantics { contentDescription = "Volume Slider" }
            )

            Text(
                text = "Value: ${sliderValue.toInt()}",
                modifier = Modifier
                    .align(Alignment.CenterHorizontally)
                    .semantics { contentDescription = "Slider Value: ${sliderValue.toInt()}" }
            )

            Divider(modifier = Modifier.padding(vertical = 16.dp))

            // Radio Buttons Section
            Text(
                text = "Radio Buttons",
                fontSize = 18.sp,
                fontWeight = FontWeight.Bold,
                modifier = Modifier.padding(bottom = 8.dp)
            )

            Column(modifier = Modifier.selectableGroup()) {
                RadioOption(
                    selected = radioValue == 1,
                    onClick = { radioValue = 1 },
                    label = "Choice A"
                )
                RadioOption(
                    selected = radioValue == 2,
                    onClick = { radioValue = 2 },
                    label = "Choice B"
                )
                RadioOption(
                    selected = radioValue == 3,
                    onClick = { radioValue = 3 },
                    label = "Choice C"
                )
            }

            Divider(modifier = Modifier.padding(vertical = 16.dp))

            // State Display Card
            Card(modifier = Modifier.fillMaxWidth()) {
                Column(modifier = Modifier.padding(16.dp)) {
                    Text(
                        text = "Current State:",
                        fontWeight = FontWeight.Bold
                    )
                    Spacer(modifier = Modifier.height(8.dp))
                    val stateText = """
                        Checkbox 1: $checkbox1
                        Checkbox 2: $checkbox2
                        Switch 1: $switch1
                        Switch 2: $switch2
                        Slider: ${sliderValue.toInt()}
                        Radio: $radioValue
                    """.trimIndent()
                    Text(
                        text = stateText,
                        modifier = Modifier.semantics {
                            contentDescription = "Current State: $stateText"
                        }
                    )
                }
            }
        }
    }
}

@Composable
private fun LabeledCheckbox(
    checked: Boolean,
    onCheckedChange: ((Boolean) -> Unit)?,
    label: String,
    description: String,
    enabled: Boolean = true
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 4.dp)
            .semantics { contentDescription = description },
        verticalAlignment = Alignment.CenterVertically
    ) {
        Checkbox(
            checked = checked,
            onCheckedChange = onCheckedChange,
            enabled = enabled
        )
        Spacer(modifier = Modifier.width(8.dp))
        Text(text = label)
    }
}

@Composable
private fun LabeledSwitch(
    checked: Boolean,
    onCheckedChange: (Boolean) -> Unit,
    label: String,
    description: String
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .padding(vertical = 4.dp)
            .semantics { contentDescription = description },
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.SpaceBetween
    ) {
        Text(text = label)
        Switch(
            checked = checked,
            onCheckedChange = onCheckedChange
        )
    }
}

@Composable
private fun RadioOption(
    selected: Boolean,
    onClick: () -> Unit,
    label: String
) {
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .selectable(selected = selected, onClick = onClick)
            .padding(vertical = 4.dp)
            .semantics { contentDescription = label },
        verticalAlignment = Alignment.CenterVertically
    ) {
        RadioButton(
            selected = selected,
            onClick = onClick
        )
        Spacer(modifier = Modifier.width(8.dp))
        Text(text = label)
    }
}
