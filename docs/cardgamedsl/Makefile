.PHONY: docs diagrams

DOC_OUT = ./architecture/build
DIAG_SRC = architecture/diagrams

docs: diagrams
	latexmk -pdf -output-directory=$(DOC_OUT) architecture/architecture.tex

# This outputs all .puml files directly into $(DOC_OUT)
diagrams:
	@mkdir -p $(DOC_OUT)
	plantuml "$(DIAG_SRC)/*.puml" -o "$(abspath $(DOC_OUT))"