import 'package:flutter/material.dart';
import 'package:flutter/rendering.dart';

void main() {
  // Enable semantics for debugging (needed for UI automation)
  WidgetsFlutterBinding.ensureInitialized();
  SemanticsBinding.instance.ensureSemantics();

  runApp(const MobileUseTestApp());
}

class MobileUseTestApp extends StatelessWidget {
  const MobileUseTestApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Mobile Use Test App',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(seedColor: Colors.blue),
        useMaterial3: true,
      ),
      home: const HomePage(),
      routes: {
        '/buttons': (context) => const ButtonsPage(),
        '/inputs': (context) => const InputsPage(),
        '/lists': (context) => const ListsPage(),
        '/forms': (context) => const FormsPage(),
      },
    );
  }
}

/// Home page with navigation to test pages
class HomePage extends StatelessWidget {
  const HomePage({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Mobile Use Test'),
        backgroundColor: Theme.of(context).colorScheme.inversePrimary,
      ),
      body: Padding(
        padding: const EdgeInsets.all(16.0),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Semantics(
              label: 'Test Pages Header',
              child: const Text(
                'Test Pages',
                style: TextStyle(fontSize: 24, fontWeight: FontWeight.bold),
              ),
            ),
            const SizedBox(height: 16),
            _NavButton(
              label: 'Buttons & Taps',
              route: '/buttons',
              icon: Icons.touch_app,
            ),
            const SizedBox(height: 8),
            _NavButton(
              label: 'Text Inputs',
              route: '/inputs',
              icon: Icons.keyboard,
            ),
            const SizedBox(height: 8),
            _NavButton(
              label: 'Scrollable Lists',
              route: '/lists',
              icon: Icons.list,
            ),
            const SizedBox(height: 8),
            _NavButton(
              label: 'Form Controls',
              route: '/forms',
              icon: Icons.check_box,
            ),
          ],
        ),
      ),
    );
  }
}

class _NavButton extends StatelessWidget {
  final String label;
  final String route;
  final IconData icon;

  const _NavButton({
    required this.label,
    required this.route,
    required this.icon,
  });

  @override
  Widget build(BuildContext context) {
    return Semantics(
      button: true,
      label: label,
      child: ElevatedButton.icon(
        onPressed: () => Navigator.pushNamed(context, route),
        icon: Icon(icon),
        label: Text(label),
        style: ElevatedButton.styleFrom(
          padding: const EdgeInsets.symmetric(vertical: 16),
        ),
      ),
    );
  }
}

/// Page for testing tap, double-tap, long-press
class ButtonsPage extends StatefulWidget {
  const ButtonsPage({super.key});

  @override
  State<ButtonsPage> createState() => _ButtonsPageState();
}

class _ButtonsPageState extends State<ButtonsPage> {
  String _lastAction = 'None';
  int _tapCount = 0;
  bool _isEnabled = true;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Buttons & Taps'),
        backgroundColor: Theme.of(context).colorScheme.inversePrimary,
      ),
      body: Padding(
        padding: const EdgeInsets.all(16.0),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            // Status display
            Card(
              child: Padding(
                padding: const EdgeInsets.all(16.0),
                child: Column(
                  children: [
                    Semantics(
                      label: 'Last Action: $_lastAction',
                      child: Text(
                        'Last Action: $_lastAction',
                        style: const TextStyle(fontSize: 18),
                      ),
                    ),
                    const SizedBox(height: 8),
                    Semantics(
                      label: 'Tap Count: $_tapCount',
                      child: Text(
                        'Tap Count: $_tapCount',
                        style: const TextStyle(fontSize: 18),
                      ),
                    ),
                  ],
                ),
              ),
            ),
            const SizedBox(height: 24),

            // Simple tap button
            Semantics(
              button: true,
              label: 'Tap Me Button',
              child: ElevatedButton(
                onPressed: () {
                  setState(() {
                    _lastAction = 'Single Tap';
                    _tapCount++;
                  });
                },
                child: const Text('Tap Me'),
              ),
            ),
            const SizedBox(height: 16),

            // Double tap area
            Semantics(
              label: 'Double Tap Area',
              child: GestureDetector(
                onDoubleTap: () {
                  setState(() {
                    _lastAction = 'Double Tap';
                    _tapCount += 2;
                  });
                },
                child: Container(
                  padding: const EdgeInsets.all(24),
                  decoration: BoxDecoration(
                    color: Colors.blue.shade100,
                    borderRadius: BorderRadius.circular(8),
                  ),
                  child: const Center(
                    child: Text('Double Tap Area'),
                  ),
                ),
              ),
            ),
            const SizedBox(height: 16),

            // Long press area
            Semantics(
              label: 'Long Press Area',
              child: GestureDetector(
                onLongPress: () {
                  setState(() {
                    _lastAction = 'Long Press';
                    _tapCount += 5;
                  });
                },
                child: Container(
                  padding: const EdgeInsets.all(24),
                  decoration: BoxDecoration(
                    color: Colors.green.shade100,
                    borderRadius: BorderRadius.circular(8),
                  ),
                  child: const Center(
                    child: Text('Long Press Area'),
                  ),
                ),
              ),
            ),
            const SizedBox(height: 16),

            // Enabled/Disabled button
            Row(
              children: [
                Expanded(
                  child: Semantics(
                    button: true,
                    label: _isEnabled ? 'Enabled Button' : 'Disabled Button',
                    child: ElevatedButton(
                      onPressed: _isEnabled
                          ? () {
                              setState(() {
                                _lastAction = 'Enabled Button Tapped';
                              });
                            }
                          : null,
                      child: Text(_isEnabled ? 'Enabled Button' : 'Disabled Button'),
                    ),
                  ),
                ),
                const SizedBox(width: 16),
                Semantics(
                  label: 'Toggle Button Enable',
                  child: Switch(
                    value: _isEnabled,
                    onChanged: (value) {
                      setState(() {
                        _isEnabled = value;
                      });
                    },
                  ),
                ),
              ],
            ),
            const SizedBox(height: 16),

            // Reset button
            Semantics(
              button: true,
              label: 'Reset Counter',
              child: OutlinedButton(
                onPressed: () {
                  setState(() {
                    _lastAction = 'Reset';
                    _tapCount = 0;
                  });
                },
                child: const Text('Reset Counter'),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

/// Page for testing text input and clear
class InputsPage extends StatefulWidget {
  const InputsPage({super.key});

  @override
  State<InputsPage> createState() => _InputsPageState();
}

class _InputsPageState extends State<InputsPage> {
  final _usernameController = TextEditingController();
  final _passwordController = TextEditingController();
  final _emailController = TextEditingController();
  final _searchController = TextEditingController();
  String _submittedData = '';

  @override
  void dispose() {
    _usernameController.dispose();
    _passwordController.dispose();
    _emailController.dispose();
    _searchController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Text Inputs'),
        backgroundColor: Theme.of(context).colorScheme.inversePrimary,
      ),
      body: SingleChildScrollView(
        padding: const EdgeInsets.all(16.0),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            // Username field
            Semantics(
              label: 'Username Input',
              textField: true,
              child: TextField(
                controller: _usernameController,
                decoration: const InputDecoration(
                  labelText: 'Username',
                  hintText: 'Enter username',
                  prefixIcon: Icon(Icons.person),
                  border: OutlineInputBorder(),
                ),
              ),
            ),
            const SizedBox(height: 16),

            // Password field
            Semantics(
              label: 'Password Input',
              textField: true,
              child: TextField(
                controller: _passwordController,
                obscureText: true,
                decoration: const InputDecoration(
                  labelText: 'Password',
                  hintText: 'Enter password',
                  prefixIcon: Icon(Icons.lock),
                  border: OutlineInputBorder(),
                ),
              ),
            ),
            const SizedBox(height: 16),

            // Email field
            Semantics(
              label: 'Email Input',
              textField: true,
              child: TextField(
                controller: _emailController,
                keyboardType: TextInputType.emailAddress,
                decoration: const InputDecoration(
                  labelText: 'Email',
                  hintText: 'Enter email',
                  prefixIcon: Icon(Icons.email),
                  border: OutlineInputBorder(),
                ),
              ),
            ),
            const SizedBox(height: 16),

            // Search field with clear button
            Semantics(
              label: 'Search Input',
              textField: true,
              child: TextField(
                controller: _searchController,
                decoration: InputDecoration(
                  labelText: 'Search',
                  hintText: 'Search...',
                  prefixIcon: const Icon(Icons.search),
                  suffixIcon: IconButton(
                    icon: const Icon(Icons.clear),
                    onPressed: () {
                      _searchController.clear();
                    },
                    tooltip: 'Clear Search',
                  ),
                  border: const OutlineInputBorder(),
                ),
              ),
            ),
            const SizedBox(height: 24),

            // Submit button
            Semantics(
              button: true,
              label: 'Submit Button',
              child: ElevatedButton(
                onPressed: () {
                  setState(() {
                    _submittedData = '''
Username: ${_usernameController.text}
Email: ${_emailController.text}
Search: ${_searchController.text}
''';
                  });
                },
                child: const Text('Submit'),
              ),
            ),
            const SizedBox(height: 16),

            // Clear all button
            Semantics(
              button: true,
              label: 'Clear All Button',
              child: OutlinedButton(
                onPressed: () {
                  _usernameController.clear();
                  _passwordController.clear();
                  _emailController.clear();
                  _searchController.clear();
                  setState(() {
                    _submittedData = 'All fields cleared';
                  });
                },
                child: const Text('Clear All'),
              ),
            ),
            const SizedBox(height: 16),

            // Display submitted data
            if (_submittedData.isNotEmpty)
              Card(
                child: Padding(
                  padding: const EdgeInsets.all(16.0),
                  child: Semantics(
                    label: 'Submitted Data: $_submittedData',
                    child: Text(_submittedData),
                  ),
                ),
              ),
          ],
        ),
      ),
    );
  }
}

/// Page for testing scroll and swipe
class ListsPage extends StatelessWidget {
  const ListsPage({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Scrollable Lists'),
        backgroundColor: Theme.of(context).colorScheme.inversePrimary,
      ),
      body: ListView.builder(
        itemCount: 50,
        itemBuilder: (context, index) {
          return Semantics(
            label: 'List Item ${index + 1}',
            child: ListTile(
              leading: CircleAvatar(
                child: Text('${index + 1}'),
              ),
              title: Text('List Item ${index + 1}'),
              subtitle: Text('Description for item ${index + 1}'),
              trailing: const Icon(Icons.chevron_right),
              onTap: () {
                ScaffoldMessenger.of(context).showSnackBar(
                  SnackBar(
                    content: Text('Tapped item ${index + 1}'),
                    duration: const Duration(seconds: 1),
                  ),
                );
              },
            ),
          );
        },
      ),
    );
  }
}

/// Page for testing checkboxes, switches, and other form controls
class FormsPage extends StatefulWidget {
  const FormsPage({super.key});

  @override
  State<FormsPage> createState() => _FormsPageState();
}

class _FormsPageState extends State<FormsPage> {
  bool _checkbox1 = false;
  bool _checkbox2 = true;
  bool _checkbox3 = false;
  bool _switch1 = false;
  bool _switch2 = true;
  double _slider = 50;
  int _radioValue = 1;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Form Controls'),
        backgroundColor: Theme.of(context).colorScheme.inversePrimary,
      ),
      body: SingleChildScrollView(
        padding: const EdgeInsets.all(16.0),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // Checkboxes
            const Text(
              'Checkboxes',
              style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold),
            ),
            CheckboxListTile(
              title: const Text('Option 1'),
              subtitle: const Text('Unchecked by default'),
              value: _checkbox1,
              onChanged: (value) {
                setState(() {
                  _checkbox1 = value ?? false;
                });
              },
            ),
            CheckboxListTile(
              title: const Text('Option 2'),
              subtitle: const Text('Checked by default'),
              value: _checkbox2,
              onChanged: (value) {
                setState(() {
                  _checkbox2 = value ?? false;
                });
              },
            ),
            CheckboxListTile(
              title: const Text('Option 3 (Disabled)'),
              subtitle: const Text('Cannot be changed'),
              value: _checkbox3,
              onChanged: null,
            ),
            const Divider(),

            // Switches
            const Text(
              'Switches',
              style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold),
            ),
            SwitchListTile(
              title: const Text('Toggle 1'),
              subtitle: const Text('Off by default'),
              value: _switch1,
              onChanged: (value) {
                setState(() {
                  _switch1 = value;
                });
              },
            ),
            SwitchListTile(
              title: const Text('Toggle 2'),
              subtitle: const Text('On by default'),
              value: _switch2,
              onChanged: (value) {
                setState(() {
                  _switch2 = value;
                });
              },
            ),
            const Divider(),

            // Slider
            const Text(
              'Slider',
              style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold),
            ),
            Semantics(
              label: 'Volume Slider',
              slider: true,
              value: '${_slider.round()}',
              child: Slider(
                value: _slider,
                min: 0,
                max: 100,
                divisions: 10,
                label: _slider.round().toString(),
                onChanged: (value) {
                  setState(() {
                    _slider = value;
                  });
                },
              ),
            ),
            Center(
              child: Semantics(
                label: 'Slider Value: ${_slider.round()}',
                child: Text('Value: ${_slider.round()}'),
              ),
            ),
            const Divider(),

            // Radio buttons
            const Text(
              'Radio Buttons',
              style: TextStyle(fontSize: 18, fontWeight: FontWeight.bold),
            ),
            RadioListTile<int>(
              title: const Text('Choice A'),
              value: 1,
              groupValue: _radioValue,
              onChanged: (value) {
                setState(() {
                  _radioValue = value ?? 1;
                });
              },
            ),
            RadioListTile<int>(
              title: const Text('Choice B'),
              value: 2,
              groupValue: _radioValue,
              onChanged: (value) {
                setState(() {
                  _radioValue = value ?? 2;
                });
              },
            ),
            RadioListTile<int>(
              title: const Text('Choice C'),
              value: 3,
              groupValue: _radioValue,
              onChanged: (value) {
                setState(() {
                  _radioValue = value ?? 3;
                });
              },
            ),
            const Divider(),

            // Status display
            Card(
              child: Padding(
                padding: const EdgeInsets.all(16.0),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    const Text(
                      'Current State:',
                      style: TextStyle(fontWeight: FontWeight.bold),
                    ),
                    Text('Checkbox 1: $_checkbox1'),
                    Text('Checkbox 2: $_checkbox2'),
                    Text('Switch 1: $_switch1'),
                    Text('Switch 2: $_switch2'),
                    Text('Slider: ${_slider.round()}'),
                    Text('Radio: $_radioValue'),
                  ],
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}
