.PHONY: push_book

push_book:
	mdbook build && \
	cd book && \
	git init && \
	git branch -M gh-pages && \
	git add -A && \
	git commit -m 'Pages' && \
	git remote add origin git@github.com:dblanovschi/robespierre && \
	git push origin gh-pages -f