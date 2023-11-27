package main

import (
	"C"
	"bytes"
	"fmt"
	"os"
	"time"

	"github.com/metacubex/geo/convert"
	"github.com/metacubex/geo/encoding/v2raygeo")

//export v2site_to_sing
func v2site_to_sing() {
	var (
		buffer   bytes.Buffer
		filename = "geosite.db"
	)
	fmt.Println("➕Loading file: geosite.dat",)
	fileContent, err := os.ReadFile("./geosite.dat")
	if err != nil {
		panic(err)
	}
	buffer.Grow(8 * 1024 * 1024) // 8 MiB
	fmt.Println("🔁Converting GeoSite database: v2ray -> sing-box")
	startTime := time.Now()

	var geositeList []*v2raygeo.GeoSite
	geositeList, err = v2raygeo.LoadSite(fileContent)
	if err != nil {
		panic(err)
	}
	
	err = convert.V2RaySiteToSing(geositeList, &buffer)
	if err != nil {
		panic(err)
	}

	err = os.WriteFile(filename, buffer.Bytes(), 0o666)
	if err != nil {
		panic(err)
	}
	fmt.Println("🎉Successfully converted to", filename, "in", time.Now().Sub(startTime))
}

func main() {}
