#!/usr/bin/env python3
"""
Download Wokwi CLI for Windows
"""

import urllib.request
import ssl
import os
import sys

def download_wokwi_cli():
    """Download Wokwi CLI executable."""
    url = "https://github.com/wokwi/wokwi-cli/releases/download/v0.16.0/wokwi-cli-win32-x64.exe"
    output_file = "wokwi-cli.exe"
    
    print("🔽 Downloading Wokwi CLI from GitHub releases...")
    print(f"URL: {url}")
    print(f"Output: {output_file}")
    
    try:
        # Create SSL context that doesn't verify certificates (for corporate networks)
        ssl_context = ssl.create_default_context()
        ssl_context.check_hostname = False
        ssl_context.verify_mode = ssl.CERT_NONE
        
        # Download the file
        with urllib.request.urlopen(url, context=ssl_context) as response:
            with open(output_file, 'wb') as f:
                f.write(response.read())
        
        # Check if file was downloaded
        if os.path.exists(output_file):
            file_size = os.path.getsize(output_file)
            print(f"✅ Wokwi CLI downloaded successfully!")
            print(f"File size: {file_size:,} bytes")
            
            # Test the executable
            print("🧪 Testing Wokwi CLI...")
            os.system(f"{output_file} --version")
            
            return True
        else:
            print("❌ File was not created")
            return False
            
    except Exception as e:
        print(f"❌ Error downloading Wokwi CLI: {e}")
        print("You can manually download from: https://github.com/wokwi/wokwi-cli/releases")
        return False

if __name__ == "__main__":
    download_wokwi_cli() 