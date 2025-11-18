package main

import (
  "context"
  "fmt"

  rbacp "voc-go-backend/internal/infrastructure/persistence/rbac"
  "voc-go-backend/internal/infrastructure/db"
)

func main() {
  cfg := db.LoadConfigFromEnv()
  pg, err := db.NewPostgres(cfg)
  if err != nil { panic(err) }
  defer pg.Close()

  repo := rbacp.NewPgMenuRepository(pg)
  ms, err := repo.ListByRoleID(context.Background(), 1)
  fmt.Println("err:", err)
  fmt.Printf("menus: %+v\n", ms)
}

