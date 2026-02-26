package com.example.mobileusetest.compose.theme

import android.os.Build
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color

// Blue color scheme matching Flutter's ColorScheme.fromSeed(seedColor: Colors.blue)
private val LightColorScheme = lightColorScheme(
    primary = Color(0xFF1565C0),          // Blue 800
    onPrimary = Color.White,
    primaryContainer = Color(0xFFBBDEFB), // Blue 100 - matches Flutter inversePrimary
    onPrimaryContainer = Color(0xFF0D47A1),
    secondary = Color(0xFF625B71),
    onSecondary = Color.White,
    secondaryContainer = Color(0xFFE8DEF8),
    onSecondaryContainer = Color(0xFF1D192B),
    background = Color(0xFFFFFBFE),
    onBackground = Color(0xFF1C1B1F),
    surface = Color(0xFFFFFBFE),
    onSurface = Color(0xFF1C1B1F),
    surfaceVariant = Color(0xFFE7E0EC),
    onSurfaceVariant = Color(0xFF49454F),
    outline = Color(0xFF79747E),
)

@Composable
fun MobileUseTestTheme(content: @Composable () -> Unit) {
    MaterialTheme(
        colorScheme = LightColorScheme,
        content = content
    )
}
