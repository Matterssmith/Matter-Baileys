// Package contract exposes the read-only WhatsApp-Matter Rust wire contract.
// No access to WhatsApp sessions, private implementation or Vault.
// Créditos: Hecho por Lournyx Matterssmith [Lou-Nyx].
package contract

import (
	"encoding/json"
	"errors"
)

const Version = 1

type Request struct {
	Version   int    `json:"version"`
	Operation string `json:"operation"`
	Provider  string `json:"provider"`
}

type Descriptor struct {
	Name              string   `json:"name"`
	Runtime           string   `json:"runtime"`
	OriginalEcosystem string   `json:"original_ecosystem"`
	Status            string   `json:"status"`
	Operations        []string `json:"operations"`
	ControlAuthority  bool     `json:"control_authority"`
}

type Response struct {
	Version  int         `json:"version"`
	OK       bool        `json:"ok"`
	Provider *Descriptor `json:"provider,omitempty"`
	Error    string      `json:"error,omitempty"`
}

func DescribeRequest(provider string) (Request, error) {
	switch provider {
	case "whatsapp-rust", "baileys", "whatsmeow":
		return Request{Version: Version, Operation: "describe", Provider: provider}, nil
	default:
		return Request{}, errors.New("unsupported_provider")
	}
}

func ParseResponse(data []byte) (*Descriptor, error) {
	if len(data) == 0 || len(data) > 8192 {
		return nil, errors.New("invalid_response_size")
	}
	var response Response
	if err := json.Unmarshal(data, &response); err != nil {
		return nil, err
	}
	if response.Version != Version || !response.OK || response.Provider == nil {
		return nil, errors.New("bridge_denied")
	}
	p := response.Provider
	if p.Runtime != "Rust" || p.ControlAuthority {
		return nil, errors.New("untrusted_descriptor")
	}
	if _, err := DescribeRequest(p.Name); err != nil {
		return nil, err
	}
	return p, nil
}
