
Collision finding is the only one that actually takes advantage of the sorted property.



#
let X be the random variable destrbing whether or not two bots collide.
Becuase the two bots are indenentantly placed in the word uniformly, we can say:
P(X)=(bot_width/dim_x)^2*(bot_height/dim_y)^2
Because X only has the values true or false, E[X]=P(X).

let Y be the random variable describing the number of bots colliding.
So Y=X1+X2+X3..Xn where n is the number of pairs of bots.
Or Y=X*(n choose 2) where n is the number of bots.
Each of these events is independant, and identical to each other.
Exploiting the linearity property of the expected value, 
E[Y]=E[X*(n choose 2)]=(n choose 2)*E[X]

What we want to find is the average number of bots colliding for a particular point
in the 2d space (lets call this L). So we simply divide the number of bots colliding by the area.
E[L]=E[Y]/(dim_x*dim_y)

E[L]=(n choose 2)*E[X]/(dim_x*dim_y)
E[L]=(n choose 2)*( (bot_width/dim_x)^2*(bot_height/dim_y)^2 ) /( dim_x*dim_y)
Simplifying this much further is hard.
Let as make an assumption that the word and the bots are square.
E[L]=(n choose 2)*( (bot_width/dim_x)^2*(bot_width/dim_x)^2 ) /(dim_x*dim_x)
E[L]=(n choose 2)*( (bot_width/dim_x)^4 ) /(dim_x^2)
E[L]=(n choose 2)* bot_width^4 / dim_x^2

So now if we fix any of the 3 variabes, we can calculate the third.
Lets solve for the dim.

dim_x^2=(n choose 2)* bot_width^4 /E[L]
dim_x=sqrt((n choose 2)* bot_width^4 /E[L])


 Now lets sanity check.
 If we plug in:
 n=100*100.
 bot_width=1;
 E[L]=1;
 we should get 100 back.

# Space and Time Complexity

I dont know what the theoretical average time compleity of this algorithm would be. The performance depends so wildly on the distribution of the position and sizes of the bots that are fed into it. And in more usecases, there would be certain patterns to the input data. For example, in most cases, I would hope that the bots are mostly not intersecting, (because presumably the user is using this system to keep the bots apart). And another presumption might be that size of the bounding boxes would tend to be small relative to the world in which all the bots live. 

In the average case, if you thought of all the bots as nodes, and then added edges to the nodes whose bots intersected, you'd hope that your graph was planar. This might be another way of figuring out the time complexity. The number of edges of a planar graph is bounded from above by 3*v-6. This is much smaller than the naive v*v edges of a complete graph.

That said bounding it by the worst case is easy, because in the worst case every single bot is colliding with every other bot. So the worst case is that all the bots are directly ontop of each other. Then the tree nor the mark and sweep algorithm could take any adantage of the situation and it would degenerate into the naive algorithm.

In the best case, all the bots live in only leaf nodes, and none of the bots intersect. Interestingly by the pigeon principle, if you have more bots than there are leaf nodes then this best case scenario isnt possible. And this is the case. We are picking the height of the tree such that every leaf node will have a specific amount of bots. We also know that every non leaf node will most likely have at least one bot in it since it was used as the median. The non leaf nodes that dont have any bots in them, must not have any because none of its children have bots either.

# Epsilon

Before we analyze the rebalance and query algorithms, lets come up with an approximation of how often bots would intersect a divider. Lets first look at the root. If you had a bunch of bots randomly and uniformly distrubuted in a 2d space, how many of them would intersect the median divider? The answer to this depends on the sizes of the bots. If all the bots were points, then hopefully only one bot would intersect with the divider. The only case this wouldnt be true is if multiple bots had the same x position as the median bot. If we're talking about real numbers, then I think the likelyhood of two bots randomly sharing the exact same x value is next to impossible. Since we are not dealing with real numbers, its more likely. On some bounded interval, there are only so many values that a floating point can have inbetween them, and even less so for integers. But it would still be a small enough chance that we can ignore. So for the cases where the bot is a point, I think its safe to say that epsilon is around 1 for the root.

As the sizes of the bots increases, epsilon would also grow. By how much I'm not sure. But thats not the real concern. We are only concerned about the complexity as n grows. We can just assume that the bot size is constant, whatever it may be. 
For our purposes, its simpler to just think of the bots as points since it doesnt effect our n complexity.

So the question is as n grows, how is episolon effected?

It clearly must also grow somewhat. The more bots there are, the greater the likelyhood that any bot will have the same value as the median bot.  
So we have:
1/x + 1/x +1/x +1/x + ... =  n/x
where x is the possible x values.


probability a bot intersects median:p(bot)=d/s
let random variable be whether or not bot touches x. So it either is or it isnt.
It happens with probability d/s. It doesnt happen with probably 1-d/s.
So E[X]=1*d/s+0*(1-d/s)=d/s.

Expected value is additive. so E[x1+x2+..]=E[x1]+E[x2]+E[x3]...

so we have: (nd)/s=expected value of each bot touching.

So that is just for the root. For the other levels, I couldn't come up with a nice closed form equation. Just recursve ones that depend on the acencetors. So lets just assume that each level is just as bad as the root. In reality, each level would have less bots to consider. 

So we're saying that for any level, we expect  (n*d)/s to intersect the divider.

Again we are assuming uniform distribution.

So below algorithms are only efficient if d<<s. 

So e=(n*d)/s.  and we assume d << s.


# construction time complexity

Lets looking at rebalancing. For each node, there are three steps that need to be done:
binning process
sort middile
recurse left,right

As you go down the tree, less time is spent binning, and more time is spent sorting.

at the root, binning would have an input of N, and sorting would take e1 (the amount intersecting the divider. The hope is that this is  asmall number).
at the second level, binning would have an input of (N-e1), and sorting would take 2*e2, so if we write this out:


level1  =  1*bin(n)+sort(e)

level2  =  2*(bin((n-1*e)/2)+sort(e/2))

level3  =  4*(bin((n-2*e)/4)+sort(e/4)) 

level4  =  8*(bin((n-4*e)/8)+sort(e/8))

The total running time is the sum of all of these.

Sorting is done using rust's built in sorting, which has big(o) of log(n)*n like any sorting algoritm.
So for the purposes of finding the O(n) we can replace sort(n) with log(n)*n.
The binning process first finds the median at each level using pdqselect which has an average running time of O(n). 
Once it finds the median, it then binns all the bots into three bins. This is also O(n).
So for the purposes of finding the O(n) we can replace bin(n) with simply n.

First lets just replace the binning:

level1  =  1*n+sort(e)

level2  =  2*(n-1*e)/2+sort(e/2))

level3  =  4*(n-2*e)/4+sort(e/4)) 

level4  =  8*(n-4*e)/8+sort(e/8))

Lets split them into two series. One for binning and one for sorting.

complete binning running time = 1*n + 2*(n-1*e)/2 + 4*(n-2*e)/4 + 8*(n-4*e)/8 + .... 

= n + n-1*e + n-2*e  + n-4*e + ...

= (n + n + n + n + ...) - e( 1 + 2 + 4 + 8 + ..)

lets assume the tree height is h.

= n*h - e*(sum(2^k,0,h))

= n*h - e*(1-2^h)/-1

= n*h - e*(-1 + 2^h)

= n*h + e-2^h 

complete sorting running time = 1*sort(e) + 2*sort(e/2) + 4*sort(e/4) + 8*sort(e/8) + ...

=e*log(e) + 2*(e/2)*log(e/2) + 4*(e/4)*log(e/4) + 8*(e/8)*log(e/8) + ...

=e*log(e) + e*log(e/2) + e*log(e/4) + e*log(e/8) + ...

=e*(log(e) + log(e/2) + log(e/4) + log(e/8) + ...)

=e*(log(e) + log(e) - log(2) + log(e) - log(4) + log(e) - log(8) + ...)

=e*(h*log(e) -log(2)-log(4) -log(8) - ... )

=e*(h*log(e) - (log(2)+log(4)+log(8)+...)  )

=e*(h*log(e) - (log(2*4*8...)     ) )

=e*(h*log(e) - log(2^h))

=h*e*log(e) - e*log(2^h)

So complete running time is binning and sorting complete times combined

(n*h + e-2^h) + (h*e*log(e) - h*e*log(2))

= n*h + e + h*e*log(e) - 2^h - h*e*log(2)

Notice how many terms depend on e. The hope is that e is very small to bein with. 
The dominating term is n*h. We choose the height of the complete tree based off of the number of bots,
so we can replace h with log(n) leaving n*log(n) for the dominating term. 

So assuming e is small, we running time of creating the tree is n*log(n). 



# collision pair query time complexity


Lets make some sweeping (no pun intended) assumptions. Every node has around the same number of bots,
and we will call it e (same as rebalancing)

Level 1: from the root we have to recurse all the way down until we visit all nodes that touch the root divider.
	sweep(e)+bjsweep(e)*h
level 2:
	sweep(e)*2+bjsweep(e)*(h-1)*2
level 3:
	sweep(e)*4+bjsweep(e)*(h-2)*4

so we have:

(se+be*h) + 2*(se+be*(h-1)) + 2^2(se+be*(h-2)) + ...

Lets split it into two terms

(se+2*se+4*se+....)+(be*h  + 2*be*(h-1)+4*be(h-2)+..)

now lets distribute:

se*(2^0+2^1+2^2+2^3+...)   + be*(1h +2(h-1)+4(h-2)+8(h-3)+...)
                                 


The first term is a geometric series, so is equal to:
se*(2^h-1)
or roughly:
se*(2^h)

The second term, is more complicated, but a geometric series can be broken off and you are left with a summation
over ia^i. After some simplifying the second term is close to:
be(h*2^h)

I'm dropping small constants left and right since we only care about the complexity at a large scale.

so we have:
se*(2^h)+be(h*2^h)

here:
2^h(se+be*h);

We want to bound it by a function that takes n as input, not h.

2^(log2(n/10))*(se+be*log2(n/10))

(n/10)(se+be*log2(n/10))

So I think the complexity of the querying is also O(n*log2(n)), but it is clearly more expensive than rebalancing.



So overall we have to functions that bound complexity:

rebalance_cost=(bin(n)+sort(e))*log2(n/bots_per_node);
query_cost=(n/bots_per_node)(sweep_sort(e)+bi_sweep(e)*log2(n/bots_per_node))

The bijective sweep algorithm is less expensive than the sweep sort. The sweep sort algorithm has to check every body against every other in theslice that is passed to it. By contrast, the bi_sweep() algorithm checks two slices of bots only against each other. 



# Space Complexity


The space complexity, on the other hand, is much easier to figure out. 
The height of the tree is=log2(num/10).
The num of nodes=2^height.
So the number of nodes as a function of bots is nodes=2^(log2(num/10))=num/10.
So the number of nodes created is linear to the number of bots.
So I think space complexity is O(n).
