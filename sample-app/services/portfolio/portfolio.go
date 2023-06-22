package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"log"
	"net/http"
)

type Account struct {
	AccountNum string `json:"accountNum"`
}

type Pricing struct {
	Ticker string `json:"ticker"`
	Price float32 `json:"price"`
}

func main() {
	log.SetFlags(log.LstdFlags | log.Lshortfile)
	http.HandleFunc("/totalAsset", TotalAsset)
	log.Println("Portfolio service started ...")
	log.Fatal(http.ListenAndServe(":8001", nil))
}

func TotalAsset(w http.ResponseWriter, r *http.Request) {
	var account Account
	// get the json body and decode into account
	err := json.NewDecoder(r.Body).Decode(&account)
	if err != nil {
		w.WriteHeader(http.StatusBadRequest)
		log.Println("Invalid account number: ", err.Error())
		return
	}

	// retrieve all the positions
	// for the simplicity, our example account only holds 
	// 100 shares of 1 stock (BNS)

	shares := 100

	payload := []byte(`{"ticker": "BNS"}`)
	// call the Pricing service to get the price for each ticker
	req, err := http.NewRequest(http.MethodPost, 
						"http://pricing",
						// local test
						// "http://localhost:8002",
						bytes.NewReader(payload))
	if err != nil {
		log.Println("Failed to create a HTTP request: " + err.Error())
		w.WriteHeader(http.StatusInternalServerError)
		return
	}

	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("x-app-name", "portfolio")
	client := &http.Client{}

	res, err := client.Do(req)
	if err != nil {
		log.Println("Failed to get a response from the Pricing service: " + err.Error())
		w.WriteHeader(http.StatusInternalServerError)
		return 
	}

	// Pass all the response headers from the Pricing to our client
	for key, element := range res.Header {
		w.Header().Add(key, element[0])
	}

	defer res.Body.Close()
	var pricing Pricing
	err = json.NewDecoder(res.Body).Decode(&pricing)
	if err != nil {
		log.Println("Failed to retrieve the response body: " + err.Error())
		w.WriteHeader(http.StatusInternalServerError)
		return 
	}

	// pricing := Pricing{"BNS", 67.56}
	totalAsset := float32(shares) * pricing.Price
	body := []byte(`{"totalAsset": ` + fmt.Sprintf("%.2f", totalAsset) + `}`)
	w.WriteHeader(http.StatusOK)
	w.Write(body)
}
