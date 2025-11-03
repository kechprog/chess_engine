#!/usr/bin/env python3
"""
Simple screenshot tool to capture the chess engine window.
"""
import time
import sys

try:
    from PIL import ImageGrab
    import win32gui
    import win32con
except ImportError:
    print("Installing required packages...")
    import subprocess
    subprocess.check_call([sys.executable, "-m", "pip", "install", "pillow", "pywin32"])
    from PIL import ImageGrab
    import win32gui
    import win32con

def find_window_by_title(title_substring):
    """Find window handle by title substring."""
    def callback(hwnd, windows):
        if win32gui.IsWindowVisible(hwnd):
            window_title = win32gui.GetWindowText(hwnd)
            if title_substring.lower() in window_title.lower():
                windows.append((hwnd, window_title))
        return True

    windows = []
    win32gui.EnumWindows(callback, windows)
    return windows

def capture_window(hwnd):
    """Capture screenshot of specific window."""
    # Bring window to foreground
    win32gui.SetForegroundWindow(hwnd)
    time.sleep(0.2)  # Give it time to come to front

    # Get window dimensions
    rect = win32gui.GetWindowRect(hwnd)
    x, y, x2, y2 = rect

    # Capture the window
    screenshot = ImageGrab.grab(bbox=(x, y, x2, y2))
    return screenshot

def main():
    print("Looking for chess engine window...")

    # Wait a moment for window to appear
    time.sleep(0.5)

    # Find the chess window specifically
    windows = find_window_by_title("chess")

    if windows:
        for hwnd, title in windows:
            if title.strip().lower() == "chess":
                print(f"Found chess window: {title}")
                screenshot = capture_window(hwnd)
                filename = "chess_window.png"
                screenshot.save(filename)
                print(f"Screenshot saved as: {filename}")
                print(f"Size: {screenshot.size}")
                return

    print("Chess window not found in exact match, capturing all matching windows...")

    # Also capture full screen
    print("Capturing entire screen...")
    screenshot = ImageGrab.grab()
    filename = "fullscreen.png"
    screenshot.save(filename)
    print(f"\nScreenshot saved as: {filename}")
    print(f"Size: {screenshot.size}")

if __name__ == "__main__":
    main()
