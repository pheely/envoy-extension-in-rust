package main

import (
	"encoding/json"
	"fmt"
	"log"
	"math/rand"
	"net/http"
)

func main() {
	log.SetFlags(log.LstdFlags | log.Lshortfile)
	http.HandleFunc("/", GetPrices)
	log.Println("Pricing service started ...")
	log.Fatal(http.ListenAndServe(":8002", nil))
}

type Quote struct {
	Ticker string `json:"ticker"`
}

func GetPrices(w http.ResponseWriter, r *http.Request) {
	var quote Quote
	err := json.NewDecoder(r.Body).Decode(&quote)
	if err != nil {
		w.WriteHeader(http.StatusBadRequest)
		log.Println("Failed to decode request: " + err.Error())
		return
	}

	log.Println("Received a request from " + r.Header.Get("x-app-name"))
	w.Header().Add("Content-Type", "application/json")
	w.WriteHeader(http.StatusOK)
	
	body := []byte(`{"ticker": "` + quote.Ticker + `", "price": ` + 
						fmt.Sprintf("%.2f", rand.Float32() * 100) + `}`)
	w.Write(body)
}