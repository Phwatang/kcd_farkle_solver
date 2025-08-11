# KCD Farkle Solver
A solver for the dice game Farkle (using the ruleset shown in the videogame Kingdom Come Deliverance).

This analysis **purely focuses on a player maximising the (expected) score** they get in a single round of Farkle. It does not focus maximising the odds of winning in a 2-player Farkle game. In otherwords, the strategies proposed here is **not the same as optimal play**.
 - For brevity sake however, any mention of "optimal strategy" going forth will be about maximising expected score gain...

For a single round of Farkle, a player could potentially perform many roll turns within the round. We will think about each turn as 2 main states:
1. The (new) turn has just begun. We know our current score and the dice set we are about to roll with.
2. We have now rolled our dice set. We know if we have gone bust or not. If we haven't gone bust we can pick dice to add to our score and decide to roll again or end the round here.
    - Deciding to roll again would lead us back to state 1.

**We will calculate** the returns of the optimal strategy **from state 1** of the Farkle game. I.e we are asking, given some dice and a current score, what will be the total expected score gain from the next roll(s) assuming we play with the optimal strategy. 

So our optimal strategy only has 2 pieces of information to consider:
 - Our dice set (and the weightings of our dice)
 - Our current score

To make the final formulas more concise, **we will initially assume that the player does not loop back to have all dice** when they've been all used up. When stated, this will be factored back in near the end of the derivations.

## Notation
To start analysing this, we first need to make a distinction between sets of dice and the potential set of outcomes from a set of dice.

We first define the dice:
 - Let $d_1, d_2,..., d_6$ represent the probability mass functions for dices 1 to 6 respectively.
 - Now let $D = \lbrace d_i: i \in \lbrace 1,2,..., 6 \rbrace \rbrace$ represent the set of all 6 starting dice.
 - Since each $d_i$ represents a distinct die, we thus treat each $d_i$ as distinct object even if they may share the same probability mass function as another. This is to ensure no $d_i$ disappear when we start forming sets.

Now we start defining outcome spaces. Given a subset of dice $\delta \subseteq D$:
 - Let $\Omega_{\delta}$ to represent the sample space of $\delta$
    - Also define $\Omega$ to be the entire possible sample space from all possible dice subsets $\delta$. I.e $\Omega =\bigcup_{\delta \in \mathcal{P}(D)}$
 - Let $\omega \in \Omega_\delta$ represent a distinct sample outcome of the diceset $\delta$
    - $\omega$ encodes information of the outcome of dice in $\delta$ but also shows what dice are missing
    - E.g if $\delta = \lbrace d_1, d_2 \rbrace$, then a possible $\omega$ could be:
      
      $$\omega_\text{example} = \lbrace d_1 = 2, d_2 = 5, d_3 = d_4 = d_5 = d_6 = \text{Missing} \rbrace$$
      
    - The powerset $\mathcal{P}(\omega)$ will have a modified definition. It will be all possible selections/retentions of dice that are present. E.g
      
      $$\lbrace d_1 = 2, d_2 = d_3 = d_4 = d_5 = d_6 = \text{Missing} \rbrace \in \mathcal{P}(\omega_\text{example})$$ \
      $$\lbrace d_2 = 5, d_1 = d_3 = d_4 = d_5 = d_6 = \text{Missing} \rbrace \in \mathcal{P}(\omega_\text{example})$$ \
      $$\lbrace d_1 = d_2 = d_3 = d_4 = d_5 = d_6 = \text{Missing} \rbrace \in \mathcal{P}(\omega_\text{example})$$ 

We will also define one useful helper functions:
 - Let $\text{Origin}: \Omega \to \mathcal{P}(D)$ be the mapping between a dice sample to the diceset it must have come from.
    - Using the $\omega_\text{example}$ example stated before then $\text{Origin}(\omega_\text{example}) = \lbrace d_1, d_2 \rbrace$

We now start defining scoring functions to be able to playout the farkle game:
 - During a turn of farkle, we roll dice and then afterwards select the dice that contribute to our score. To compute the scoring calculation, define $\text{Score}: \Omega \to \mathbb{N}$ represent the farkle score gained from a selected subset of a dice sample.
    - If $\omega$ is an "invalid" selection then $\text{Score}(\omega) = 0$
    - Using the same $\omega$ example from before, we could select just dice $d_2$. Our selection would be:
      
      $$\omega_\text{sel} = \lbrace d_2 = 5, d_1=d_3 = d_4 = d_5 = d_6 = \text{Missing} \rbrace$$ 
      
      And so the score would be $\text{Score}(\omega_\text{sel}) = 50$.

      Including both $d_1$ and $d_2$ would be an invalid selection. I.e $\text{Score}(\omega_\text{example}) = 0$

 - Let $\text{Best}:\Omega_\delta \to \mathbb{N}$ represent the greatest possible Farkle score possible out of all possible selections from a given dice sample. I.e
   
   $$\text{Best}(\omega) = \max \lbrace \text{Score}(\omega'): \omega' \in \mathcal{P}(\omega) \rbrace$$
    - If $\text{Best}(\omega) = 0$ then no valid selections can be made. In other words, $\omega$ is a "bust".

## Optimising Expected Return
We will let $\text{Optimal}(P, \delta)$ represent the expected score gain if playing optimally.
 - $P$ - The current score for this round
 - $\delta$ - The set of dice (and their associated probability mass functions) to roll with

Note that $\text{Optimal}(P, \delta)$ represents expected score **gain**. In otherwords, if a player is currently within a round with a score of $P$, set $\delta$ dice remaining and is using this strategy, their **expected final score** for the round would be:

$$\mathbb{E}(P_\text{final}) = P + \text{Optimal}(P, \delta)$$

To further clarify, at the start of a round, the expected score at the end of the round would be $\text{Optimal}(0, D)$.

For each sample $\omega \in \Omega_\delta$ of the dice $\delta$, **assuming we have not busted on the roll**, we have 2 possible strategies:
 - $\text{Terminate}$ - Selecting dice and not to roll again
 - $\text{Hold}$ - Selecting dice and deciding to roll again

We will now analyse the expected score gain from these two choices.

### Maximising $\text{Terminate}$
To maximise decision $\text{Terminate}$, we simply want to select dice to create the highest scoring dice hands. In otherwords:

$$\text{Terminate}  = \max \lbrace \text{Score}(\omega'): \omega' \in \mathcal{P}(\omega) \rbrace = \text{Best}(\omega)$$

### Maximising $\text{Hold}$
When choosing dice, we have to keep in mind that the dice we don't choose also affect our expected scores going into the future rolls.

Assuming we are using the same optimal strategy going into the next roll, the expected score gain of decision $\text{Hold}$ is:
```math
\begin{align*}
\text{Hold} = & \max \lbrace \\
& \quad \text{Score}(\omega') + 
\text{Optimal}(P + \text{Score}(\omega'), \delta \backslash \text{Origin}(w')): \omega' \in \mathcal{P}(\omega)
\\ &\rbrace
\end{align*}
```

### Combining Together
Obviously now the optimal strategy to to simply select the higher of option $\text{Terminate}$ or $\text{Hold}$ for all samples $\omega$. We also need to subtract off the expected loss from busting. This means:

$$\text{Optimal}(P, \delta) = \mathbb{E}(\max \lbrace \text{Terminate}, \text{Hold} \rbrace ) - \mathbb{P}(\text{Best}(\omega) = 0 ~\text{for}~ \omega \in \Omega_\delta)P$$

We now see that to compute $\text{Optimal}(P, \delta)$, we need to compute $\text{Hold}$. However $\text{Hold}$ requires us to compute $\text{Optimal}(...)$ again but for differing values of $P$ and $\delta$.

We also note that when the player has no dice left ($\delta = \emptyset$), due to all the dice being used to form scoring hands, the player wraps back round to having all 6 dice again to roll with. Therefore:

$$\text{Optimal}(P, \emptyset) = \text{Optimal}(P, D)$$

 - With how $\text{Hold}$ and $\text{Terminate}$ have been defined, this does cause a logical contradiction. However, I've defined things in this order to make things concise. $\text{Hold}$ and $\text{Terminate}$ can easilly be redefined to fix the contradiction.

Clearly then, we need some starting datapoints for the output of $\text{Optimal}(...)$ to build out from. We do not have these however **leading us to a bootstrapping problem**.

## Iterating Up
To solve this bootstrap problem, we will define a new strategy $\text{Optimal}_n$. This is defined as: The strategy that maximises expected score gain **assuming we must end our turn in the next $n$ rolls**.

Obviously as $n \to \infty$, we should expect $\text{Optimal}_n$ and $\text{Optimal}$ to give the same results.

To analyse this strategy, we can use the same option $\text{Terminate}$ and $\text{Hold}$ approach as before:
 - Option $\text{Terminate}$ stays the same
 - Option $\text{Hold}$ is slightly tweaked. Going into the next roll we would have used up $1$ roll in our $n$ turn budget. Therefore
```math
\begin{align*}
\text{Hold}_n = & \max \lbrace \\
& \quad \text{Score}(\omega') + 
\text{Optimal}_{n-1}(P + \text{Score}(\omega'), \delta \backslash \text{Origin}(w')): \omega' \in \mathcal{P}(\omega)
\\ &\rbrace
\end{align*}
```

We now also realise the trivial case of $\text{Optimal}_1$. In this case we only have 1 roll to use and thus option $\text{Terminate}$ is only available. This means the associated expected score gain from this is just:
```math
\begin{align*}
\text{Optimal}_1(P, \delta) &= \mathbb{E}(\text{Terminate}) - \mathbb{P}(\text{Best}(\omega) = 0 ~\text{for}~ \omega \in \Omega_\delta)P \\
&= \mathbb{E}(\text{Best}) - \mathbb{P}(\text{Best}(\omega) = 0 ~\text{for}~ \omega \in \Omega_\delta)P
\end{align*}
```
 - This is simple to compute for all possible $P$ and $\delta$ values combinations

With $\text{Optimal}_1$ being solved, we can begin to solve for $\text{Optimal}_2$, $\text{Optimal}_3$, etc using the recurrence relation:
```math
\begin{align*}
\text{Optimal}_n(P, \delta) &= \mathbb{E}(\max \lbrace \text{Terminate}, \text{Hold}_n \rbrace ) - \mathbb{P}(\text{Best}(\omega) = 0 ~\text{for}~ \omega \in \Omega)P \\
&= \mathbb{E}(
\max \lbrace \\
& \quad \quad \text{Best}(\omega), \\
& \quad \quad \max \lbrace \text{Score}(\omega') + 
\text{Optimal}_{n-1}(P + \text{Score}(\omega'), \delta \backslash \text{Origin}(w')): \omega' \in \mathcal{P}(\omega) \rbrace \\
& \quad \rbrace : \omega \in \Omega_\delta) - \mathbb{P}(\text{Best}(\omega) = 0 ~\text{for}~ \omega \in \Omega_\delta)P
\end{align*}
```

Again we also need to factor in the player wrapping back round to all $6$ dice when all dice being have been used. Therefore, for all $n$:

$$\text{Optimal}_n(P, \emptyset) = \text{Optimal}_n(P, D)$$

## Computational Feasibility
Even with the iterative method described before, computing a $\text{Optimal}_n$ seems practically impossible at first glance.

### Too Many $P$ and $\delta$?
One issue with this approach is we can only compute the output for one combination of $P$ and $\delta$ at a time. Depending on the amount of possible values of each, this approach becomes infeasible.

Analysing $P$ possibilities:
 - With our version of Farkle, the maximum score we could ever get in a single round is maximum score possible in the entire game. This would be $6000$.
    - $6000$ itself doesn't need to be considered however since the player would have won the round at that point
 - Each scoring dice face combination gives out a score that is a multiple of $50$
 - This gives us $120$ distinct values for $P$

Analysing $\delta$ possibilities:
 - $\delta$ is simply a subset of $D$ which is 6 dice.
 - This gives us $2^6=64$ distinct values for $\delta$

Combining these two together, a full computation of $\text{Optimal}_n$ will require $120 \times 64 =7680$ outputs to compute.

### Too Many $\omega$?
Another issue is computing expectations. Each output of $\text{Optimal}_n$ requires performing an expectation calculation. A single expectation requires going through all possible samples in the relevant sample space.

Our sample space $\Omega_\delta$ is dictated by $\delta$. In the worst case scenario, $\delta$ will contain all 6 dice. I.e $\delta = D$. This leaves us with a maximum of $6^6=46656$ possibilities in $\Omega_\delta$.

### Too Many $w'$?
When performing option $\text{Hold}$, we must find the best selection given the roll sample that appears. Since we are choosing dice, the worst case number of selections possible is when we have 6 dice to choose from. This results in $2^6=64$ selection possibilities

### Total Computation
Upon first glance, assuming that $\text{Optimal}_{n-1}$ is known, computing $\text{Optimal}_n$ will require at maximum $7680 \times 6^6 \times 2^6 = 2.29 \times 10^{10}$ "computations". This is obviously pretty bad but with modern computing, each computation of $\text{Optimal}_n$ is reasonably quick. (About 10min single threaded with my pc).

### Slight Improvement
Computations can be reduced by storing some calculations. In particular, we could precompute all relevant values of the payoff for $\text{Hold}$ before calculating $\text{Optimal}_n$.

$\text{Hold}$ requires knowing the current score $P$ and the current dice roll sample $\omega$.
 - We know $P$ is only $120$ distinct values
 - For an $\omega$, each dice can be $\text{Missing}$ or be a value between $1$ and $6$. Thus $\omega$ can have $7^6 = 117649$
 - Once we include in all the $2^6$ possibilities in selection, this leaves $120 \times 7^6 \times 2^6 = 903,544,320$ computations

Assuming that $\text{Hold}$ has been calculated beforehand, to calculate $\text{Optimal}_n$, we only have to roll through $P$, $\delta$ and $\omega$ sample possibilities. This would entail $120 \times 2^6 \times 6^6 = 358,318,080$ computations. If we add the computations required for $\text{Hold}$ we reach a final computation count of $1.261 \times 10^9$

## Findings
For a starting score of 0 and with all dice present (i.e start of a new round), we had the following payoffs:
 - Optimal_1 = 399
 - Optimal_2 = 517.9492
 - Optimal_3 = 517.9492

This suggests that looking only two moves ahead is enough to maximise expected score.

## To Do
To do list if I get around to it:
 - Find another approach to calculate optimal score
    - Allows for cross comparing
 - Tidy up presentation of findings
 - Implement more robust checkpoint system
