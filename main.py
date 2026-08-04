import sys
import os

# Optional: Add local directory to path to ensure imports work correctly
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from ui import App

if __name__ == "__main__":
    app = App()
    app.mainloop()
