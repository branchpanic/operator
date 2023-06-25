import("stdfaust.lib");

process = env * os.osc(440 * 2)
    with {
        env = button("trigger") : ba.impulsify : en.ar(0.001, 0.002);
    };
