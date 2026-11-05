#!/usr/bin/awk -f
# Converts apps/v4/registry/themes.ts into src/theme/theme_data.rs
# Usage: awk -f gen_theme_data.awk themes.ts > theme_data.rs

BEGIN {
  n = split("background foreground card card-foreground popover popover-foreground primary primary-foreground secondary secondary-foreground muted muted-foreground accent accent-foreground destructive border input ring chart-1 chart-2 chart-3 chart-4 chart-5 radius sidebar sidebar-foreground sidebar-primary sidebar-primary-foreground sidebar-accent sidebar-accent-foreground sidebar-border sidebar-ring", order, " ");
  nslots = 41;
  nthemes = 0;
}

# New theme block
/^[[:space:]]+name: "/ {
  name = $0;
  sub(/^[[:space:]]+name: "/, "", name);
  sub(/",[[:space:]]*$/, "", name);
  nthemes++;
  theme[nthemes] = name;
  mode = "";
  next;
}

/^[[:space:]]+light: \{/ { mode = "light"; next; }
/^[[:space:]]+dark: \{/ { mode = "dark"; next; }

# key: "value" pairs inside a mode block
/^[[:space:]]+"?[a-z0-9-]+"?: "/ {
  if (mode != "") {
    line = $0;
    sub(/^[[:space:]]+/, "", line);
    key = line;
    sub(/"?: .*$/, "", key);
    gsub(/"/, "", key);
    # Take the final quoted string on the line (handles quoted and bare keys).
    # Key by theme name: emission happens in END, after the whole file is read.
    if (match(line, /"[^"]*"[[:space:]]*,?[[:space:]]*$/)) {
      val = substr(line, RSTART + 1, RLENGTH - 2);
      sub(/"[[:space:]]*$/, "", val);
      vals[name "," mode "," key] = val;
    }
  }
  next;
}

END {
  print "// Generated from the shadcn/ui source (apps/v4/registry/themes.ts) by";
  print "// src/theme/gen_theme_data.awk. Do not edit the data by hand; regenerate instead.";
  print "//";
  print "// Slot order matches `colors::SLOT_NAMES` (41 slots).";
  print "use super::colors::Slots;";
  print "";
  print "/// One named theme from the source registry.";
  print "pub struct ThemeSpec {";
  print "    /// Registry name, e.g. `\"neutral\"` or `\"blue\"`.";
  print "    pub name: &'static str,";
  print "    /// True for the 7 base colors with full palettes (neutral, stone, zinc,";
  print "    /// mauve, olive, mist, taupe); false for sparse accent themes.";
  print "    pub is_base_color: bool,";
  print "    /// Light-mode slots (empty string = inherit).";
  print "    pub light: Slots,";
  print "    /// Dark-mode slots (empty string = inherit).";
  print "    pub dark: Slots,";
  print "}";
  print "";
  for (t = 1; t <= nthemes; t++) {
    name = theme[t];
    isbase = (vals[name "," "light,background"] != "") ? "true" : "false";
    const = toupper(name);
    gsub(/-/, "_", const);
    printf "pub const %s: ThemeSpec = ThemeSpec {\n    name: \"%s\",\n    is_base_color: %s,\n", const, name, isbase;
    printf "    light: [\n";
    for (i = 1; i <= nslots; i++) {
      v = vals[name "," "light," order[i]];
      if (v == "") v = "";
      printf "        \"%s\",\n", v;
    }
    printf "    ],\n";
    printf "    dark: [\n";
    for (i = 1; i <= nslots; i++) {
      v = vals[name "," "dark," order[i]];
      if (v == "") v = "";
      printf "        \"%s\",\n", v;
    }
    printf "    ],\n};\n\n";
  }
  print "pub static THEMES: &[ThemeSpec] = &[";
  for (t = 1; t <= nthemes; t++) {
    const = toupper(theme[t]);
    gsub(/-/, "_", const);
    printf "    &%s,\n", const;
  }
  print "];";
}
