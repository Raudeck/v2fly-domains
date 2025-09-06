package main

/*
#cgo CFLAGS: -g -Wall
#include <stdio.h>
#include <stdlib.h>
#include <errno.h>
*/
import "C"

import (
	"os"
	"unsafe"

	"github.com/sagernet/sing-box/common/srs"
	"github.com/sagernet/sing-box/option"
	"github.com/sagernet/sing/common/json"
)

//export compileRuleset
func compileRuleset(text *C.char, len C.int, version C.uchar, outputPath *C.char, outputPathLen C.int) *C.char {
	output := C.GoStringN(outputPath, outputPathLen)
	content := C.GoBytes(unsafe.Pointer(text), len)
	plainRuleset, err := json.UnmarshalExtended[option.PlainRuleSetCompat](content)
	if err != nil {
		return C.CString(err.Error())
	}
	outputFile, err := os.Create(output)
	if err != nil {
		return C.CString(err.Error())
	}
	err = srs.Write(outputFile, plainRuleset.Options, uint8(version))
	if err != nil {
		outputFile.Close()
		os.Remove(output)
		return C.CString(err.Error())
	}
	outputFile.Close()
	return nil
}

func main() {}
