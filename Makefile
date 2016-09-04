SRCDIR=${CURDIR}

alfred-chrome : alfred-chrome.swift frameworks
	xcrun -sdk macosx swiftc -F Carthage/Build/Mac/ alfred-chrome.swift && install_name_tool -add_rpath "@executable_path/Frameworks" alfred-chrome

frameworks:
	rm -rf Frameworks && ln -s Carthage/Build/Mac Frameworks

install: alfred-chrome
	cd ~/Library/Application\ Support/Alfred\ 3/Alfred.alfredpreferences/workflows/user.workflow.${WORKFLOW} && \
	rm -rf alfred-chrome Frameworks/* && \
	cp ${SRCDIR}/alfred-chrome . && mkdir -p Frameworks && cp -a ${SRCDIR}/Carthage/Build/Mac/*.framework Frameworks

clean:
	rm -rf Frameworks alfred-chrome

default: alfred-chrome
