* show relative scale to (universe -> ... -> plank length)
* infinite depth use increasing presision when zooming (f32 -> f64 -> i64 -> i128 -> [u8]) (or use rug?)
* NOTE: we cannot change parents, because what if we are zooming exactly at the center? there is no common parant but the root node.
* other fractal types
* custom fractal formulas?
* merge fractal types to create hybrid fractals
* non fractal types? acutal procedural universe explorer with accurate scales.
    * see: https://orteil.dashnet.org/nested
* save and load locations as simple strings (to clipboard)
    * can be shared as a url
    * probably something simple and universal as quadtree path + offset
    * this url shuld gneerate a nice preview on redded (use open graph, https://css-tricks.com/essential-meta-tags-social-media/)

* Idea: make core platform independend.
