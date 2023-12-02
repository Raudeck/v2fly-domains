package main

/*
	typedef const char cchar_t;
*/
import "C"

import (
	"bytes"
	"fmt"
	"os"
	"time"
	"io"
	"bufio"
	"strings"
	"path/filepath"

	"github.com/metacubex/geo/convert"
	"github.com/metacubex/geo/encoding/v2raygeo"
	"github.com/sagernet/sing-box/common/geosite"
	"github.com/sagernet/sing-box/common/srs"
	"github.com/sagernet/sing-box/constant"
	"github.com/sagernet/sing-box/option"
	"github.com/sagernet/sing/common"

	"github.com/v2fly/v2ray-core/v5/app/router/routercommon"
	"google.golang.org/protobuf/proto"
)

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

func parse(vGeositeData []byte) (map[string][]geosite.Item, error) {
	vGeositeList := routercommon.GeoSiteList{}
	err := proto.Unmarshal(vGeositeData, &vGeositeList)
	if err != nil {
			return nil, err
	}
	domainMap := make(map[string][]geosite.Item)
	for _, vGeositeEntry := range vGeositeList.Entry {
			code := strings.ToLower(vGeositeEntry.CountryCode)
			domains := make([]geosite.Item, 0, len(vGeositeEntry.Domain)*2)
			attributes := make(map[string][]*routercommon.Domain)
			for _, domain := range vGeositeEntry.Domain {
					if len(domain.Attribute) > 0 {
							for _, attribute := range domain.Attribute {
									attributes[attribute.Key] = append(attributes[attribute.Key], domain)
							}
					}
					switch domain.Type {
					case routercommon.Domain_Plain:
							domains = append(domains, geosite.Item{
									Type:  geosite.RuleTypeDomainKeyword,
									Value: domain.Value,
							})
					case routercommon.Domain_Regex:
							domains = append(domains, geosite.Item{
									Type:  geosite.RuleTypeDomainRegex,
									Value: domain.Value,
							})
					case routercommon.Domain_RootDomain:
							if strings.Contains(domain.Value, ".") {
									domains = append(domains, geosite.Item{
											Type:  geosite.RuleTypeDomain,
											Value: domain.Value,
									})
							}
							domains = append(domains, geosite.Item{
									Type:  geosite.RuleTypeDomainSuffix,
									Value: "." + domain.Value,
							})
					case routercommon.Domain_Full:
							domains = append(domains, geosite.Item{
									Type:  geosite.RuleTypeDomain,
									Value: domain.Value,
							})
					}
			}
			domainMap[code] = common.Uniq(domains)
			for attribute, attributeEntries := range attributes {
					attributeDomains := make([]geosite.Item, 0, len(attributeEntries)*2)
					for _, domain := range attributeEntries {
							switch domain.Type {
							case routercommon.Domain_Plain:
									attributeDomains = append(attributeDomains, geosite.Item{
											Type:  geosite.RuleTypeDomainKeyword,
											Value: domain.Value,
									})
							case routercommon.Domain_Regex:
									attributeDomains = append(attributeDomains, geosite.Item{
											Type:  geosite.RuleTypeDomainRegex,
											Value: domain.Value,
									})
							case routercommon.Domain_RootDomain:
									if strings.Contains(domain.Value, ".") {
											attributeDomains = append(attributeDomains, geosite.Item{
													Type:  geosite.RuleTypeDomain,
													Value: domain.Value,
											})
									}
									attributeDomains = append(attributeDomains, geosite.Item{
											Type:  geosite.RuleTypeDomainSuffix,
											Value: "." + domain.Value,
									})
							case routercommon.Domain_Full:
									attributeDomains = append(attributeDomains, geosite.Item{
											Type:  geosite.RuleTypeDomain,
											Value: domain.Value,
									})
							}
					}
					domainMap[code+"@"+attribute] = common.Uniq(attributeDomains)
			}
	}
	return domainMap, nil
}
//export generate
func generate(geositeInputFile *C.cchar_t, ruleSetOutputDir *C.cchar_t) {
	file, err := os.Open(C.GoString(geositeInputFile))
	if err != nil {
		fmt.Print(err)
		os.Exit(1) 
	}
	defer file.Close()
	stat, err := file.Stat();
	if err != nil {
		fmt.Print(err)
		return 
	}
	bs := make([]byte, stat.Size())
	_, err = bufio.NewReader(file).Read(bs)
	if err != nil && err != io.EOF {
		fmt.Print(err)
		return 
	}
	domainMap, err := parse(bs)
	if err != nil {
		fmt.Print(err)
		return 
	}
	err = os.MkdirAll(C.GoString(ruleSetOutputDir), 0o755)
	if err != nil {
		fmt.Print(err)
		return 
	}
	for code, domains := range domainMap {
			var headlessRule option.DefaultHeadlessRule
			defaultRule := geosite.Compile(domains)
			headlessRule.Domain = defaultRule.Domain
			headlessRule.DomainSuffix = defaultRule.DomainSuffix
			headlessRule.DomainKeyword = defaultRule.DomainKeyword
			headlessRule.DomainRegex = defaultRule.DomainRegex
			var plainRuleSet option.PlainRuleSet
			plainRuleSet.Rules = []option.HeadlessRule{
					{
							Type:           constant.RuleTypeDefault,
							DefaultOptions: headlessRule,
					},
			}
			srsPath, _ := filepath.Abs(filepath.Join(C.GoString(ruleSetOutputDir), "geosite-"+code+".srs"))
			os.Stderr.WriteString("write " + srsPath + "\n")
			outputRuleSet, err := os.Create(srsPath)
			if err != nil {
				fmt.Print(err)
				return 
			}
			err = srs.Write(outputRuleSet, plainRuleSet)
			if err != nil {
				outputRuleSet.Close()
				fmt.Print(err)
				return 
			}
			outputRuleSet.Close()
	}
}

func main() {}
