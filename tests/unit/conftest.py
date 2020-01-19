import os
import sys

# add path to ThavalonWeb to sys.path, for tests to successfully import from game
sys.path.append(os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))