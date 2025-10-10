package main

import "testing"

func TestGreet(t *testing.T) {
	got := greet("Forge")
	want := "Hello, Forge!"

	if got != want {
		t.Fatalf("greet(\"Forge\") = %q; want %q", got, want)
	}
}
