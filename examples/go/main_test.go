package main

import "testing"


func TestGreetBasic(t *testing.T) {
	got := greet("Forge")
	want := "Hello, Forge!"

	if got != want {
		t.Fatalf("greet(\"Forge\") = %q; want %q", got, want)
	}
}


func TestGreetTable(t *testing.T) {
	tests := []struct {
		name string
		want string
	}{
		{"Forge", "Hello, Forge!"},
		{"Alice", "Hello, Alice!"},
		{"Bob", "Hello, Bob!"},
	}

	for _, tt := range tests {
		got := greet(tt.name)
		if got != tt.want {
			t.Errorf("greet(%q) = %q; want %q", tt.name, got, tt.want)
		}
	}
}

func TestGreetEmpty(t *testing.T) {
	got := greet("")
	want := "Hello, !"

	if got != want {
		t.Fatalf("greet(\"\") = %q; want %q", got, want)
	}
}
