# Signals, Power, and Networking — Physical Layer Concepts

---

## Fiber Optic

Instead of pushing electrons down copper, fiber sends light pulses down a glass strand. On = 1, off = 0 at the most basic level — but real systems are more sophisticated (see modulation below).

**Why photons beat electrons for long distance:**
- **Attenuation** — copper loses signal every few km, needs repeaters. Fiber loses very little, repeaters every ~100km. This is the big one for undersea cables.
- **Interference** — electrons are affected by nearby EM fields. Photons don't care.
- **Bandwidth** — light's carrier frequency is in the hundreds of terahertz range. More frequency range available = more capacity.

**Total internal reflection** — how light stays in the fiber. The core is surrounded by cladding with a slightly different refractive index. Light that hits the boundary at a shallow angle bounces back in instead of escaping. Light zigzags down the fiber and can't escape as long as you don't bend it too sharply.

**WDM (wavelength division multiplexing)** — you can run multiple wavelengths (colors) of light down the same fiber simultaneously, each carrying an independent data stream. A prism-like device splits them back out at the receiver. One fiber strand can carry 80+ wavelengths at 100 Gbps each — that's 8 Tbps down one strand of glass thinner than a human hair.

**Polarization** — light also has orientation (horizontal/vertical). Modern systems use this too, running two independent streams on orthogonal polarizations down the same fiber.

**Why copper is still in undersea cables** — fiber can't carry power. The glass fiber handles data, but a copper conductor runs alongside it to power the repeater amplifiers every ~100km. Even with all of fiber's advantages, there's no avoiding copper for energy.

---

## Modulation — Encoding Data into Waves

A wave has physical properties that exist regardless of whether we use them:
- **Amplitude** — height of the wave
- **Phase** — where the wave is in its cycle
- **Frequency** — how fast it oscillates
- **Polarization** — orientation of the wave

We look at those properties, decide they're distinguishable at the receiver, and assign bit patterns to them. Completely arbitrary mapping — as long as sender and receiver agree.

The simplest scheme is **OOK (on-off keying)** — light on = 1, light off = 0. One bit per pulse.

Real systems use **QAM (quadrature amplitude modulation)** — encoding information in both amplitude and phase simultaneously. Instead of 1 bit per pulse, you encode 4, 6, or 8 bits per symbol by having many distinct amplitude+phase combinations. More bits per symbol = more data without needing faster switching.

**The tradeoff:** higher-order QAM requires more precise detection. Noise blurs amplitude levels together. If two levels are too similar, the receiver misreads them. This is Shannon's limit from another angle — more bits per symbol requires better signal-to-noise ratio.

**Same principles apply everywhere** — WiFi, 5G, DSL, fiber. All variations of modulating a carrier wave to pack more bits per symbol. The physics gives you knobs to turn, you decide what the knobs mean, and Shannon sets the ceiling on how many knobs you can have and how finely you can tune them.

---

## Undersea Cables

The internet backbone is mostly physical fiber optic cables on the ocean floor. When you ping a US server from Japan, you're sending light pulses through a cable on the Pacific floor.

**Speed of light in fiber** is ~67% the speed of light in vacuum. The Pacific is ~9000km wide — even at the physical limit you're looking at ~45ms one way. Real routes are longer with switching overhead, so ~100ms+ round trip is basically unavoidable.

**How they're laid** — a specialized cable-laying ship carries the cable on giant spools, paying it out from the stern as it sails the route. Deep ocean is the easy part. Hard parts:
- **Shore landings** — transitioning from deep water to a beach without damage
- **Repairs** — a ship has to grapple for the cable on the seafloor, pull it up, splice it, re-lay it. Breaks happen from earthquakes and anchors in shallow water.
- **Power** — repeaters every ~100km need power. A copper conductor runs the length of the cable carrying ~10,000V DC from shore just to power them.

**Cable construction** — heavily armored near shore (steel wire exterior, threat from anchors/trawlers). In deep ocean it slims down to roughly garden-hose diameter — mostly just fiber core and polyethylene jacket.

---

## AC vs DC

**Why AC won for power grids:**

The real reason isn't efficiency — it's transformers. To transmit power over long distances you want very high voltage and low current (P = IV — same power, less heat lost to resistance). You step voltage up for transmission, then step it back down at the destination.

Transformers only work on AC — they rely on a changing magnetic field. DC just saturates the core. So AC won the grid because of transformers.

**Why electronics run on DC:**

Transistors, chips, logic circuits all need a stable voltage rail. AC would just be noise to them.

**The full picture:**

Everything comes in as AC from the wall. Anything that needs DC runs it through:
1. **Rectifier** — converts AC → DC (bumpy)
2. **Smoothing capacitors** — flatten out the ripple
3. **Voltage regulator** — keep it stable under varying load

That's what every power supply is: laptop brick, PC PSU, phone charger. All AC → DC converters.

**Why DC transformers don't work:**

A transformer works by induction — a changing magnetic field in the primary coil induces voltage in the secondary. DC creates a static field. Static field induces nothing. You just get a spike when you connect it, then silence.

This is why Edison lost the AC/DC war. His DC grid couldn't use transformers, so you couldn't step voltage up for transmission. You had to generate power at the voltage you'd use it at — meaning a power station every mile or so. AC could step up, transmit efficiently over long distance, step back down. Game over.

(This is also why Hollywood is in California — early film companies fled there to escape Edison's patent lawyers.)

**HVDC (high voltage DC)** — there's actually a push back toward DC for some long-distance grid transmission now. Modern power electronics can do the conversion efficiently, and DC has lower losses over very long runs than AC. The Edison vs Tesla battle is kind of being revisited at the grid level.

**Why AC motors want AC — and why logic doesn't:**

AC motors (fridge compressor, washing machine, ceiling fan) actually need the alternating direction. The flipping current creates a rotating magnetic field and the rotor chases it around — that's what makes it spin. Give it DC and it just locks in one position and hums.

Logic circuits need stable DC for a different reason. A transistor is a voltage-controlled switch with a threshold: above X volts = on, below = off. If the voltage swings positive and negative like AC, the transistor switches on and off 50 times a second in a way you didn't ask for — completely trashing computation. The capacitor smoothing holds voltage steady between peaks so chips never see the swing.

So:
- **Motors** want AC because alternating direction creates rotation
- **Logic** wants DC because it needs a stable reference voltage for reliable on/off switching

**Examples — what actually runs on AC vs DC:**

Devices that use AC directly:
- Fridge compressor (non-inverter)
- Washing machine drum motor (non-inverter)
- Ceiling fan
- Fluorescent tube lights
- Electric stove heating element (resistive — doesn't care about direction, just heats)
- Incandescent bulb (same — just heats a filament)
- Non-inverter window AC unit
- Microwave magnetron
- Vacuum cleaner (universal motor)
- Electric water heater element

Devices that convert AC → DC internally:
- Laptop
- Phone charger
- LED lights (driver inside converts AC→DC)
- TV
- Desktop PC
- WiFi router
- Raspberry Pi / microcontrollers
- Electric car battery charging
- USB anything
- Inverter fridge/AC unit — converts to DC, then synthesizes variable-frequency AC for the motor

The AC list is shrinking. LEDs replaced incandescents, inverter motors are replacing direct AC motors. The wall is still AC because of transmission, but more devices do their own conversion internally now.

---

## Last Mile Problem

The internet backbone is fiber. But the connection from the nearest exchange to your house is usually still copper — coax or twisted pair originally built for voice calls.

DSL and cable internet are clever ways of squeezing bandwidth out of that last stretch of copper using the same modulation tricks (QAM etc.) on a copper medium instead of fiber.

**Fiber-to-the-home (FTTH)** is being rolled out in more places. Japan has relatively high penetration — NTT pushed it aggressively. Many apartments in Japan are already on fiber all the way to the unit.
