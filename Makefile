pkgname = lzd-rs
pkgversion = 0.1
DISTNAME = "${pkgname}-${pkgversion}"
INSTALL = install
INSTALL_PROGRAM = $(INSTALL) -m 755
INSTALL_DIR = $(INSTALL) -d -m 755
INSTALL_DATA = $(INSTALL) -m 644
SHELL = /bin/sh
CAN_RUN_INSTALLINFO = $(SHELL) -c "install-info --version" > /dev/null 2>&1
progname = target/debug/lzd

objs = src/lzd.rs


.PHONY : all install install-bin install-info install-man \
         install-strip install-compress install-strip-compress \
         install-bin-strip install-info-compress install-man-compress \
         uninstall uninstall-bin uninstall-info uninstall-man \
         doc info man check dist clean distclean

all : $(progname)

$(progname) : $(objs)
	cargo build

# prevent 'make' from trying to remake source files
$(VPATH)/configure $(VPATH)/Makefile.in $(VPATH)/doc/$(pkgname).texi : ;
MAKEFLAGS += -r
.SUFFIXES :


doc :

info : $(VPATH)/doc/$(pkgname).info

$(VPATH)/doc/$(pkgname).info : $(VPATH)/doc/$(pkgname).texi
	cd $(VPATH)/doc && $(MAKEINFO) $(pkgname).texi

man : $(VPATH)/doc/$(progname).1

$(VPATH)/doc/$(progname).1 : $(progname)
	help2man -n 'educational decompressor for the lzip format' -o $@ --no-info ./$(progname)

# Makefile : $(VPATH)/configure $(VPATH)/Makefile.in
# 	./config.status

check : all
	testsuite/check.sh testsuite $(pkgversion)

install : install-bin
install-strip : install-bin-strip
install-compress : install-bin
install-strip-compress : install-bin-strip

install-bin : all
	if [ ! -d "$(DESTDIR)$(bindir)" ] ; then $(INSTALL_DIR) "$(DESTDIR)$(bindir)" ; fi
	$(INSTALL_PROGRAM) ./$(progname) "$(DESTDIR)$(bindir)/$(progname)"

install-bin-strip : all
	$(MAKE) INSTALL_PROGRAM='$(INSTALL_PROGRAM) -s' install-bin

install-info :
	if [ ! -d "$(DESTDIR)$(infodir)" ] ; then $(INSTALL_DIR) "$(DESTDIR)$(infodir)" ; fi
	-rm -f "$(DESTDIR)$(infodir)/$(pkgname).info"*
	$(INSTALL_DATA) $(VPATH)/doc/$(pkgname).info "$(DESTDIR)$(infodir)/$(pkgname).info"
	-if $(CAN_RUN_INSTALLINFO) ; then \
	  install-info --info-dir="$(DESTDIR)$(infodir)" "$(DESTDIR)$(infodir)/$(pkgname).info" ; \
	fi

install-info-compress : install-info
	lzip -v -9 "$(DESTDIR)$(infodir)/$(pkgname).info"

install-man :
	if [ ! -d "$(DESTDIR)$(mandir)/man1" ] ; then $(INSTALL_DIR) "$(DESTDIR)$(mandir)/man1" ; fi
	-rm -f "$(DESTDIR)$(mandir)/man1/$(progname).1"*
	$(INSTALL_DATA) $(VPATH)/doc/$(progname).1 "$(DESTDIR)$(mandir)/man1/$(progname).1"

install-man-compress : install-man
	lzip -v -9 "$(DESTDIR)$(mandir)/man1/$(progname).1"

uninstall : uninstall-bin

uninstall-bin :
	-rm -f "$(DESTDIR)$(bindir)/$(progname)"

uninstall-info :
	-if $(CAN_RUN_INSTALLINFO) ; then \
	  install-info --info-dir="$(DESTDIR)$(infodir)" --remove "$(DESTDIR)$(infodir)/$(pkgname).info" ; \
	fi
	-rm -f "$(DESTDIR)$(infodir)/$(pkgname).info"*

uninstall-man :
	-rm -f "$(DESTDIR)$(mandir)/man1/$(progname).1"*

dist : doc
	ln -sf $(VPATH) $(DISTNAME)
	tar -Hustar --owner=root --group=root -cvf $(DISTNAME).tar \
	  $(DISTNAME)/AUTHORS \
	  $(DISTNAME)/COPYING \
	  $(DISTNAME)/ChangeLog \
	  $(DISTNAME)/INSTALL \
	  $(DISTNAME)/Makefile.in \
	  $(DISTNAME)/NEWS \
	  $(DISTNAME)/README \
	  $(DISTNAME)/configure \
	  $(DISTNAME)/*.cc \
	  $(DISTNAME)/testsuite/check.sh \
	  $(DISTNAME)/testsuite/test.txt \
	  $(DISTNAME)/testsuite/em.lz \
	  $(DISTNAME)/testsuite/fox.lz \
	  $(DISTNAME)/testsuite/fox_*.lz \
	  $(DISTNAME)/testsuite/test.txt.lz
	rm -f $(DISTNAME)
	lzip -v -9 $(DISTNAME).tar

clean :
	-rm -f $(progname) $(objs)

distclean : clean
	-rm -f Makefile config.status *.tar *.tar.lz
