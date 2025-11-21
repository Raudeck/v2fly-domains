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

	"github.com/metacubex/mihomo/constant/provider"
	P "github.com/metacubex/mihomo/rules/provider"
)

//export singboxCompileRuleset
func singboxCompileRuleset(text *C.char, len C.int, version C.uchar, outputPath *C.char, outputPathLen C.int) *C.char {
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

//export mihomoCompileRuleset
func mihomoCompileRuleset(text *C.char, len C.int, outputPath *C.char, outputLen C.int) *C.char {
	rulesetContent := C.GoBytes(unsafe.Pointer(text), len)
	outPath := C.GoStringN(outputPath, outputLen)
	file, err := os.OpenFile(outPath, os.O_WRONLY|os.O_CREATE|os.O_TRUNC, 0644)
	if err != nil {
		return C.CString(err.Error())
	}
	err = P.ConvertToMrs(rulesetContent, provider.Domain, provider.TextRule, file)
	if err != nil {
		return C.CString(err.Error())
	}
	return nil
}

func main() {}
