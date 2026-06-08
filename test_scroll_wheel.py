import subprocess
import time
import os
import sys

def take_screenshot(name, window_id):
    xwd_path = f"{name}.xwd"
    png_path = f"/home/soulr27/.gemini/antigravity/brain/6fac74aa-2896-48ee-9004-db75ff8b8784/{name}.png"
    print(f"Taking screenshot: {png_path}...")
    subprocess.run(["xwd", "-id", window_id, "-out", xwd_path])
    subprocess.run(["python3", "/home/soulr27/.gemini/antigravity/brain/6fac74aa-2896-48ee-9004-db75ff8b8784/scratch/xwd_to_png.py", xwd_path, png_path])
    if os.path.exists(xwd_path):
        os.remove(xwd_path)

def run_test():
    print("Launching Mouser GUI...")
    env = os.environ.copy()
    env["RUST_LOG"] = "info"
    proc = subprocess.Popen(
        ["./target/debug/mouser-rs", "--debug"],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        env=env
    )

    try:
        # Wait for window to load
        time.sleep(3.0)

        # Search for window
        print("Searching for Mouser-rs window...")
        try:
            window_id = subprocess.check_output(
                ["xdotool", "search", "--onlyvisible", "--name", "Mouser-rs"]
            ).decode().strip().split('\n')[0]
            print(f"Found Window ID: {window_id}")
        except subprocess.CalledProcessError:
            print("Error: Mouser-rs window not found!")
            proc.terminate()
            return

        # Activate window
        subprocess.run(["xdotool", "windowactivate", window_id])
        subprocess.run(["xdotool", "windowraise", window_id])
        time.sleep(1.0)

        # Move mouse to outside the card (100, 100) and click to focus safely
        print("Moving mouse to background overlay and clicking to focus...")
        subprocess.run(["xdotool", "mousemove", "--window", window_id, "100", "100"])
        time.sleep(0.5)
        subprocess.run(["xdotool", "click", "1"])
        time.sleep(1.0)

        # Move mouse to center of window (640, 360) over the list
        print("Moving mouse to list container...")
        subprocess.run(["xdotool", "mousemove", "--window", window_id, "640", "360"])
        time.sleep(0.5)

        # Simulate scroll down (button 5) three times, then scroll up (button 4) three times
        print("Simulating scroll wheel events...")
        for _ in range(5):
            subprocess.run(["xdotool", "click", "5"])
            time.sleep(0.1)
        
        time.sleep(1.0)
        take_screenshot("modal_scroll", window_id)

        for _ in range(5):
            subprocess.run(["xdotool", "click", "4"])
            time.sleep(0.1)

        time.sleep(1.0)
    finally:
        print("Terminating Mouser GUI...")
        proc.terminate()
        try:
            stdout, stderr = proc.communicate(timeout=2)
            print("--- STDOUT ---")
            print(stdout)
            print("--- STDERR ---")
            print(stderr)
        except subprocess.TimeoutExpired:
            proc.kill()
            stdout, stderr = proc.communicate()
            print("--- STDOUT ---")
            print(stdout)
            print("--- STDERR ---")
            print(stderr)

if __name__ == "__main__":
    run_test()
