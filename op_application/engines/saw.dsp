import("stdfaust.lib");

vol = hslider("volume [unit:dB]", -20, -96, 0, 0.1) : ba.db2linear : si.smoo;
freq = hslider("freq [unit:Hz]", 440, 20, 24000, 1);
gate = checkbox("gate");

process =
    (.7 * s1 + .7 * s2 + .9 * s3) * vol
with {
    env1 = en.asr(.05, 1.0, .1, gate);
    osc1 = os.sawtooth(freq) : fi.lowpass(1, 10000);

    env2 = en.asr(.1, 0.9, .4, gate);
    osc2 = os.sawtooth(freq * ba.cent2ratio(-1.5 + -10 * os.osc(10))) : fi.lowpass(1, 3000);

    env3 = en.asr(.2, 1.0, .5, gate);
    osc3 = os.sawtooth(freq * ba.semi2ratio(-24) * ba.cent2ratio(1)) : fi.lowpass(1, 600);

    s1 = env1 * osc1;
    s2 = env2 * osc2;
    s3 = env3 * osc3;
};
