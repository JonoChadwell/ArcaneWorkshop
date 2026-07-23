# Notes on Game Mechanics

Continuum of Form:
 - State of Matter
    - Solid
        - Crystal
        - Metal
        - 
    - Liquid
    - Particulate
      - Powder
      - Lumps
    - Crystal
    - Metal

Stations -- Full workstations with complicated environmental interactions
 - Garden
   - Consumes liquid and air
   - Produces plants
 - Cauldron
   - Consumes liquid, 
   - Produces ??
 - Alembic
   - Consumes liquid and heat
   - Produces liquid
 - Transmutation Circle
   - Consumes ??
   - Produces ??
   - In one configuration, the transmutation circle takes two (/n) inputs and randomly shuffles them together until they are identical.
 - Animal Pen
   - Consumes food and drink
   - Produces waste

Tools:
 - Shovel
   - Dig up dirt or plants
 - Butcher's Knife
   - Process animals from pens
 - 


Mechanics of Traces:
 - Section defunct. Save all traces, and worry about memory later.
 - An array of 256 entries of a "trace" in the Continuum.
 - Each trace records a situation and a clash of intent.
 - Each trace has a "strength", and the strength of a trace must be overcome by a clash to rewrite it. Potentially on a scale of 0-255, with 255 being "immortal".
 - When two or more substances are mixed together traces are randomly selected from each source substance in proportion to the amounts of substance of each type.
 - (not sure) When a particular trace is overwritten, some remnants of the previous trace in the slot are maintained
   - This reduces the ability to exactly match traces.
 - Example: crushing a leaf into a paste.
   - Through pestle and mortar, the alchemist applies a crushing intent to the substance.
   - Clash caused between "intent to be a leaf" and "intent to crush".
     - A slot is randomly selected on the left. That slot previously contained a record of sunlight hitting the leaf.
   - The slot is replaced by the new "crushed" record.
     - The addition of this new record "echoes" through all remaining records. The form imposed by other records weakens, which in turn weakens each other record as a whole.
   - After crushing, the generated intent of the leaf is more "intent of form mush" than "intent of form plant", though both are will still be present unless crushing continues much longer.

Every single item in the game shall be created by a specific player at a specific time. The only exceptions shall be a small number of objects spawned in by "Xah".

Specific intent interactions:
 - Intent of Form of Plant clashes with Intent of Crushing to produce Intent of Form of Mush.




There exists water that is "imbued with an intent of imparting stillness". What exactly does this mean?
 - Something in the Continuum (represented by traces in the game) of that object shapes the spirit produced by the substance such that when it decays the intent produced is imparting stillness.
 - When the water is poured over another substance they come into contact with eachother causing a clash of intents. The intent of imparting stillness clashes with the intent of stillness of the target. The result is (probably) recorded in the continuum of the object. The object now has more intent of stillness.
 - Each clash might diminish the intent of imparting stillness and might increase the intent of stillness. If left to soak, eventually the strength of the clashes is reduced far enough that no traces will be modified further.

The process of Continuum => Spirit creation => Decay into intent must not include any randomness. Randomness here would cause items to shift in appearance as they flickered between different intended forms.

Idea / example that may or may not make it into game mechanics:
 - A legendary caligrapher tries to write a legendary poem onto a mundane piece of paper with the most of their skill
 - The Coninuum (/ traces / history) of the paper is overwhelmed. Unless special precautions are taken, before the caligrapher can finish the paper will be destroyed -- its Continuum will be entirely overwritten by the caligrapher's actions and it will lose all traces of what made it paper in the first place.

Idea: Rather than a random slot, a clash of intent overwrites the slot that is most involved in the clash?

Idea / refinement: Spirit and only spirit can bring Intents into conflict. The types of conflict that occur depend on the generating Spirit.
 - Example: The spirit of a cauldron can force some Intent onto a substance within that cauldron. Once forced upon said substance the spirit of that substance might drive more internal conflicts.


# Notes V2

Taking all of the ideas above, letting them bounce around my head for several months, and trying to tie it back into game mechanics I've to the following code / gameplay structure:

##  Substance / Intent

Objects that are part of the game have Substance. Substance has fixed integer quantities (grams?). Each substance also has 'Intent' a list of integer quantities attached to said substance detailing how much of each intent that object contains. Intent quantities are soft-capped to substance quantities.

## Intent Interactions

The heart of the game. Various intents when attached to the same object (or to two interacting objects) produce complicated effects. These effects include:
 * Annihilation -- Intents that mutually destroy each other with no output
 * Domination -- Similar to annihilation, but only the intent with smaller quantity is actually lost
 * Reaction -- Intents that mutually react to become a third distinct intent
 * Budding / Division -- Intents that cause the substance to split when they react
 * Generation -- Presence of one intent might create more of a second

Each specific reaction might be a mix of the above. These reactions may happen very quickly or very slowly. These are the physical laws of the universe.

## Spirit

The glue that turns a system for managing reactions of intent into a playable game. The reason a plant is a plant is because it has a plant spirit. Likewise for a cauldron, alembic, transmutation circle, and every other interactable object in the game world.

The ability of spirit to control the simulation is extremely narrow by design. Most of the gameplay should come from the emergent Intent Conflict layer, not from shennanians at the spirit layer. Main functions of this layer are:
 1. A spirit is able to create an interaction between two substances. Interactions have types that will help shape what happens like contact, impact, tool_use, etc.
 2. A spirit is able to substitute itself out for another spirit depending on what intents it observes.
 3. If an conflict results in a division, the spirit must decide what to do with the new substance (most importantly, what spirit to give it).
 4. A spirit is able to push specific intent around through a special interaction type.

All decisions made by a spirit must be made by only looking at the intent of the attached object, and by certain game notifications like "The player used tried to use this object on that one". In that interaction the spirit held by the player as a tool will initially state whether it is capable of interacting with the target. If an interaction occurs, the spirit will decode what happens at the moment said interaction completes.

Processing tools (like a cauldron) run logic when the player puts stuff in, and may schedule follow up work to occur until they are satisfied with some outcome.

It would be very cool to encode spirit in a "code is data" format -- a little tiny program written in some artificial "spirit language" that can live and change as a means of change. This is _out of scope_ for my early goals. Spirit will be a simple enum over everything that exists, with specific source code for each spirit. Every substance has one tag.

## Encoding of Related Substances and 3d Position

The game is built upon an ECS system. Every substance will be an entity within this system, and will also have location or other control bits. Each spirit _might_ be it's own component type. TBD.


## Worked Example

A leaf plant. It grows one leaf. If the leaf is cut off it grows back. Cut it too much and the plant dies. Other plants could need water, air, or light in future iterations. A leaf plant is a simplified model. It only needs air, which it uses to make one leaf.

Because spirits operate within the ECS system, a leaf can find the air. Right now this will be one global entity initialized with some arbitrary large size.

Leaf Plant Stem:
 - Does not change size. Growing new plants comes later.
 - Has plant intent
 - "Internal" interaction
   - Must maintain homeostasis around two quantities:
     - Keep Growth intent relatively constant
     - Keep Plant intent capped (when fed by a leaf)
   - Plant intent becomes some leaf intent
   - Plant intent becomes some growth intent
   - Leaf + Growth intent produces AbsorbAir intent.

 - If and only if a leaf is missing, the spirit interacts stem + air to create a leaf. Absorbed mass goes into the leaf. All growth intent is pushed into the leaf.

Leaf Plant Leaf
 - Spirit causes a contact interaction between the leaf and the air
   - Leaf intent + Air intent creates Plant intent
   - AbsorbAir intent + Air intent adds substance to the leaf
 - Spirit pushes plant intent to stem, and pulls leaf intent from the stem.
 - "Internal" interaction
   - Leaf intent + Growth intent produces AbsorbAir intent
   - Leaf intent decays
   - Growth intent decays



How to limit the size of a leaf to 10 grams:
1. Spirit fiat -- growing is a special interaction that the spirit stops triggering.
2. Make it shrink at some rate that causes it to converge to 10
* 3. Build some intent mechanism into the leaf that smothers the reaction by which it grows.