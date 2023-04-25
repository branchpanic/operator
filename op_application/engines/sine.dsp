import("stdfaust.lib");

vol = hslider("volume [unit:dB]", -20, -96, 0, 0.1) : ba.db2linear : si.smoo;
freq = hslider("freq [unit:Hz]", 440, 20, 24000, 1);
gate = checkbox("gate");

process =
    s1 * env * vol
with {
    env = en.asr(.05, 1.0, .1, gate);
    lfo = os.osc(0.1);
    s1 = os.osc(freq);
};
