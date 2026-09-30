#! /bin/sh
# check script for Lzd - Educational decompressor for the lzip format
# Copyright (C) 2013-2025 Antonio Diaz Diaz.
#
# This script is free software: you have unlimited permission
# to copy, distribute, and modify it.

LC_ALL=C
export LC_ALL
objdir=`pwd`
testdir=`cd "$1" ; pwd`
LZIP="${objdir}"/target/debug/lzd
framework_failure() { echo "failure in testing framework" ; exit 1 ; }

if [ ! -f "${LZIP}" ] || [ ! -x "${LZIP}" ] ; then
	echo "${LZIP}: cannot execute"
	exit 1
fi

[ -e "${LZIP}" ] 2> /dev/null ||
	{
	echo "$0: a POSIX shell is required to run the tests"
	echo "Try bash -c \"$0 $1 $2\""
	exit 1
	}

if [ -d tmp ] ; then rm -rf tmp ; fi
mkdir tmp
cd "${objdir}"/tmp || framework_failure

in="${testdir}"/test.txt
in_lz="${testdir}"/test.txt.lz
em_lz="${testdir}"/em.lz
fox_lz="${testdir}"/fox.lz
fnz_lz="${testdir}"/fox_nz.lz
fail=0
test_failed() { fail=1 ; printf " $1" ; [ -z "$2" ] || printf "($2)" ; }

printf "testing lzd-%s..." "$2"

"${LZIP}" < "${in}" 2> /dev/null
[ $? = 2 ] || test_failed $LINENO

"${LZIP}" < "${in_lz}" > out || test_failed $LINENO
cmp "${in}" out || test_failed $LINENO

cat "${in}" "${in}" > in2 || framework_failure
cat "${in_lz}" "${in_lz}" | "${LZIP}" > out2 || test_failed $LINENO
cmp in2 out2 || test_failed $LINENO
rm -f out2 || framework_failure

cat "${in_lz}" "${in_lz}" > out2.lz || framework_failure
printf "\ngarbage" >> out2.lz || framework_failure
"${LZIP}" -d < out2.lz > out2 || test_failed $LINENO
cmp in2 out2 || test_failed $LINENO
rm -f in2 out2 out2.lz || framework_failure

touch empty || framework_failure
"${LZIP}" -d < "${em_lz}" > em || test_failed $LINENO
cmp empty em || test_failed $LINENO

printf "\ntesting bad input..."

cat "${em_lz}" "${em_lz}" | "${LZIP}" -d > em 2> /dev/null
[ $? = 2 ] || test_failed $LINENO
cmp empty em || test_failed $LINENO
rm -f empty em || framework_failure
cat "${em_lz}" "${in_lz}" | "${LZIP}" -d > out 2> /dev/null
[ $? = 2 ] || test_failed $LINENO
cmp "${in}" out || test_failed $LINENO
cat "${in_lz}" "${em_lz}" | "${LZIP}" -d > out 2> /dev/null
[ $? = 2 ] || test_failed $LINENO
cmp "${in}" out || test_failed $LINENO

"${LZIP}" < "${fnz_lz}" 2> /dev/null
[ $? = 2 ] || test_failed $LINENO

for i in fox_v2.lz fox_s11.lz fox_de20.lz \
         fox_bcrc.lz fox_crc0.lz fox_das46.lz fox_mes81.lz ; do
	"${LZIP}" < "${testdir}"/$i > /dev/null 2>&1
	[ $? = 2 ] || test_failed $LINENO $i
done

"${LZIP}" < "${fox_lz}" > fox || test_failed $LINENO
for i in fox_bcrc.lz fox_crc0.lz fox_das46.lz fox_mes81.lz ; do
	"${LZIP}" < "${testdir}"/$i > out 2> /dev/null
	[ $? = 2 ] || test_failed $LINENO $i
	cmp fox out || test_failed $LINENO $i
done
rm -f fox || framework_failure

cat "${in_lz}" "${in_lz}" > in2.lz || framework_failure
cat "${in_lz}" "${in_lz}" "${in_lz}" > in3.lz || framework_failure
if dd if=in3.lz of=trunc.lz bs=14682 count=1 2> /dev/null &&
   [ -e trunc.lz ] && cmp in2.lz trunc.lz ; then
# can't detect truncated header of non-first member
	for i in 6 20 14664 14688 ; do
		dd if=in3.lz of=trunc.lz bs=$i count=1 2> /dev/null
		"${LZIP}" < trunc.lz > /dev/null 2>&1
		[ $? = 2 ] || test_failed $LINENO $i
	done
else
	printf "warning: skipping truncation test: 'dd' does not work on your system."
fi
rm -f in2.lz in3.lz trunc.lz || framework_failure

cp "${in_lz}" ingin.lz || framework_failure
printf "g" >> ingin.lz || framework_failure
cat "${in_lz}" >> ingin.lz || framework_failure
"${LZIP}" -d < ingin.lz > out || test_failed $LINENO
cmp "${in}" out || test_failed $LINENO
rm -f out ingin.lz || framework_failure

echo
if [ ${fail} = 0 ] ; then
	echo "tests completed successfully."
	cd "${objdir}" && rm -r tmp
else
	echo "tests failed."
fi
exit ${fail}
