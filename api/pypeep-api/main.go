package main

import (
	"encoding/json"
	"fmt"
	"io"
	"log/slog"
	"os"
	"sync"

	"database/sql"
	"net/http"

	_ "github.com/mattn/go-sqlite3"
)

const (
	ROOT_URL   = "https://pypi.org/simple/%s/"
	DB_DRIVER  = "sqlite3"
	DB_PATH    = "pypeep.db"
	TABLE_NAME = "requirements"
)

var (
	db         *sql.DB
	httpClient *http.Client
)

type PyRequirement struct {
	id             int
	name           string
	currentVersion string
}

func loadRequirements() (names []string) {
	res, err := db.Query("SELECT name FROM requirements;")
	if err != nil {
		slog.Error("failed to load rows from requirements")
		os.Exit(-1)
	}
	defer res.Close()
	for res.Next() {
		var name string
		res.Scan(&name)
		names = append(names, name)
	}
	return
}

func fetchLatestVersion(requirementName string) (latestVersion string) {
	// Make an http request.
	url := fmt.Sprintf(ROOT_URL, requirementName)
	req, err := http.NewRequest(http.MethodGet, url, nil)
	if err != nil {
		slog.Error("encountered error on creating request")
		os.Exit(-1)
	}
	req.Header.Set("Accept", "application/vnd.pypi.simple.v1+json")
    slog.Info(fmt.Sprintf("fetching current version of %q", requirementName))
	res, err := httpClient.Do(req)
	if err != nil {
		slog.Error("encountered error on http request")
		os.Exit(-1)
	}
	defer res.Body.Close()

	// Deserialize the response.
	body, err := io.ReadAll(res.Body)
	data := make(map[string][]string)
	json.Unmarshal(body, &data)
	versions := data["versions"]
	slog.Debug(fmt.Sprintf("found the following versions %q, %q", requirementName, versions))

	// Grab the latest version.
	latestVersion = versions[len(versions)-1]
	slog.Info(fmt.Sprintf("the latest version of %q is %q", requirementName, latestVersion))
	return
}

func main() {
	slog.Info("starting")
	slog.Info("connecting http client")
	httpClient = &http.Client{}
	slog.Info("connecting database client")
	var err error
	db, err = sql.Open(DB_DRIVER, DB_PATH)
	if err != nil {
		slog.Error("failed to connect to database", DB_PATH)
	}
	defer db.Close()

	wg := new(sync.WaitGroup)
	for _, name := range loadRequirements() {
		wg.Add(1)
		go func(requirementName string) {
			defer wg.Done()
			latestVersion := fetchLatestVersion(requirementName)
            slog.Info(fmt.Sprintf("updating db record for %q", requirementName))
			db.Exec("UPDATE requirements SET current_version = ? WHERE name = ?", latestVersion, requirementName)
            slog.Info(fmt.Sprintf("updated db for %q", requirementName))
		}(name)
	}
	wg.Wait()
	slog.Info("done!")
}
