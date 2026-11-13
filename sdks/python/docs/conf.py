import sys

project = "Universal Agent Runtime Python SDK"
copyright = "2026, Prometheus AGS"
extensions = ["sphinx.ext.autodoc", "sphinx.ext.napoleon", "sphinx.ext.viewcode"]
html_theme = "furo" if sys.version_info >= (3, 11, 5) else "alabaster"
autodoc_typehints = "none"
