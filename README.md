**[查看中文说明](README_zh.md)**





# Paste Bypass Tool

## Project Overview

The Paste Bypass Tool is a command-line application written in Rust that bypasses websites and applications that disable or monitor standard paste operations. Instead of directly triggering a paste event, this tool simulates character-by-character keyboard input, avoiding paste detection mechanisms.

## Main Features

- **Clipboard Capture**: Reads the system clipboard content and stores it in an internal buffer.
- **Keyboard Input Simulation**: Simulates input of the buffered content character by character through the Windows Input API.
- **Global Hotkey Support**: Use the global hotkey Ctrl+Shift+V (although, for unknown reasons, the hotkey function may not always trigger reliably) or enter options in the command line opened by the program to trigger the paste bypass.
- **Multiple Speed Modes**: Ranging from slow to ultra-fast, suitable for different levels of security checks.
- **Batch Processing Optimization**: Handles long text efficiently, significantly reducing waiting time.
- **Custom Parameter Settings**: Allows you to customize input delay and batch size to meet specific needs.

## System Requirements

- Windows OS (Windows 10/11 recommended)
- No administrator privileges required
- Single executable file with no external dependencies

## Installation

1. Click on the release on the right to download the `paste_bypass.exe` executable file. It may be blocked by Windows Defender; if so, choose 'Keep' in Windows Defender's action options. This project is safe and can be used with confidence.
2. No installation required; just double-click to run.
3. (Optional) Place the program in your system’s PATH for convenient command-line use.

## Usage Instructions

### Basic Workflow

1. Start the program.
2. In the source application, copy text (Ctrl+C).
3. Use the global hotkey (Ctrl+Shift+V) or the program’s menu option to trigger the bypass paste.
4. Within the 3-second countdown, switch to the target input field.
5. The program will automatically simulate keyboard input, bypassing paste detection.

### Detailed Menu Options

The program provides the following main menu options:

1. **Capture Clipboard**: Reads the current clipboard content into the buffer.
2. **Simulate Input**: Simulates keyboard input of the buffered content.
3. **Show Current Buffer**: Displays the currently stored text.
4. **Set Key Delay**: Customizes the delay time between characters.
5. **Set Speed Mode**: Selects a preset input speed mode.
6. **Custom Input Parameters**: Advanced settings, including batch size.
7. **Exit**: Closes the program.

## Speed Modes Explained

The program offers four speed modes to adapt to different website detection mechanisms:

1. **Slow Mode**:
   - Single-character processing, 20ms delay per character, 300ms interval between batches
   - Suitable for websites with strict input speed detection
   - Safest but slowest
2. **Normal Mode** (default):
   - Processes 5 characters at a time, 10ms delay per character, 50ms interval between batches
   - Balances speed and security
   - Suitable for most websites
3. **Fast Mode**:
   - Processes 20 characters at a time, 2ms delay per character, 20ms interval between batches
   - Higher speed, suitable for less restrictive websites
   - Might be detected by more stringent sites
4. **Ultra-Fast Mode**:
   - Processes 50 characters at a time, no character delay, only a 10ms interval between batches
   - Highest performance and significantly reduces processing time for large text
   - Least natural typing pattern, highest detection risk

## Custom Parameter Settings

If the preset modes do not meet your needs, you can set the following via **Custom Input Parameters**:

- **Character Delay**: The wait time (in milliseconds) after each character is entered.
- **Batch Size**: The number of characters to send per batch.
- **Batch Interval**: The wait time (in milliseconds) between batches of characters.

## Technical Details

- Uses the Windows API (SendInput) to simulate keyboard input.
- Processes arrays of INPUT structures in batches to reduce API calls.
- Multi-threaded design ensures the hotkey response is not blocked.
- Unicode support, capable of handling Chinese and other special characters.

## Frequently Asked Questions

**Q: Why can some websites still detect that this isn’t real input?**
 A: Some websites use more complex detection mechanisms, such as monitoring input rhythm changes or timestamps. Try using **Slow Mode** or customizing parameters to mimic more natural input patterns.

**Q: The tool is slow when handling long text. How can I speed it up?**
 A: Use **Fast** or **Ultra-Fast** modes, or customize the batch size and delays for faster input.

**Q: The hotkey doesn’t respond. What should I do?**
 A: Make sure no other program is using the Ctrl+Shift+V hotkey. Try restarting the application or using the menu options manually.

**Q: I see garbled characters when entering Chinese or special characters.**
 A: The program supports Unicode input, but some applications may have character set compatibility issues. Try using **Slow Mode** for special characters.

## Development Information

- Language: Rust
- Main Dependency: winapi
- Compilation: `cargo build --release`

## Disclaimer

This tool is intended for legal and ethical use only, such as working around restricted text input environments for legitimate purposes. Users must comply with all applicable laws and website terms of use. The developer assumes no responsibility for any misuse of this tool.
