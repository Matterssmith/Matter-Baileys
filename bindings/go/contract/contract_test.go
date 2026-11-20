package contract

import (
	"encoding/json"
	"testing"
)

// Créditos: Hecho por Lournyx Matterssmith [Lou-Nyx].
func TestDescribe(t *testing.T) {
	r, err := DescribeRequest("whatsapp-rust")
	if err != nil || r.Operation != "describe" {
		t.Fatalf("invalid describe request: %v", err)
	}
	if _, err := DescribeRequest("../../vault"); err == nil {
		t.Fatal("accepted unsafe provider")
	}
	data, err := json.Marshal(Response{
		Version: Version, OK: true,
		Provider: &Descriptor{Name: "baileys", Runtime: "Rust", ControlAuthority: false},
	})
	if err != nil {
		t.Fatal(err)
	}
	if _, err := ParseResponse(data); err != nil {
		t.Fatal(err)
	}
}
func TestRejectAuthority(t *testing.T) {
	_, err := ParseResponse([]byte(`{"version":1,"ok":true,"provider":{"name":"whatsmeow","runtime":"Rust","control_authority":true}}`))
	if err == nil {
		t.Fatal("accepted untrusted authorization")
	}
}
